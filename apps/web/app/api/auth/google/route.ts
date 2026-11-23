import { NextResponse } from 'next/server';
import jwt from 'jsonwebtoken';
import { cookies } from 'next/headers';
import crypto from 'crypto';

const JWT_SECRET = process.env.JWT_SECRET || 'fallback_secret_for_dev';

// Name of the short-lived cookie that carries the anti-CSRF `state` value
// between the redirect to Google and the callback below.
const GOOGLE_STATE_COOKIE = 'google_oauth_state';
const STATE_COOKIE_MAX_AGE = 10 * 60; // 10 minutes — just long enough for sign-in

// Constant-time string comparison so a mismatched `state` can't be detected
// via response-time differences.
function timingSafeEqual(a: string, b: string): boolean {
  const bufA = Buffer.from(a);
  const bufB = Buffer.from(b);
  if (bufA.length !== bufB.length) return false;
  return crypto.timingSafeEqual(bufA, bufB);
}

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const code = searchParams.get('code');
  const state = searchParams.get('state');
  const cookieStore = await cookies();

  // No `code` yet: this is the request that starts the OAuth redirect.
  // Generate a random `state`, stash it in a short-lived httpOnly cookie, and
  // send the browser to Google with that same value so the callback below
  // can confirm the request round-tripped through this browser (login CSRF
  // protection).
  if (!code) {
    const generatedState = crypto.randomBytes(32).toString('hex');
    cookieStore.set(GOOGLE_STATE_COOKIE, generatedState, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
      maxAge: STATE_COOKIE_MAX_AGE,
    });

    const authorizeUrl = new URL('https://accounts.google.com/o/oauth2/v2/auth');
    authorizeUrl.searchParams.set('client_id', process.env.GOOGLE_CLIENT_ID || '');
    authorizeUrl.searchParams.set(
      'redirect_uri',
      process.env.GOOGLE_REDIRECT_URI || 'http://localhost:3000/api/auth/google'
    );
    authorizeUrl.searchParams.set('response_type', 'code');
    authorizeUrl.searchParams.set('scope', 'openid email profile');
    authorizeUrl.searchParams.set('state', generatedState);

    return NextResponse.redirect(authorizeUrl);
  }

  // Callback leg: the returned `state` must match the cookie set above
  // before we do anything with `code`. Reject (and always clear the cookie)
  // on a missing or mismatched value.
  const storedState = cookieStore.get(GOOGLE_STATE_COOKIE)?.value;
  cookieStore.delete(GOOGLE_STATE_COOKIE);

  if (!state || !storedState || !timingSafeEqual(state, storedState)) {
    return NextResponse.redirect(new URL('/auth?error=Invalid state parameter', request.url));
  }

  try {
    // Exchange auth code for ID token via Google APIs
    const tokenResponse = await fetch('https://oauth2.googleapis.com/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        client_id: process.env.GOOGLE_CLIENT_ID || '',
        client_secret: process.env.GOOGLE_CLIENT_SECRET || '',
        code,
        grant_type: 'authorization_code',
        redirect_uri: process.env.GOOGLE_REDIRECT_URI || 'http://localhost:3000/api/auth/google',
      }),
    });

    if (!tokenResponse.ok) {
      return NextResponse.redirect(new URL('/auth?error=Google OAuth failed', request.url));
    }

    const tokenData = await tokenResponse.json();
    const idToken = tokenData.id_token;

    if (!idToken) {
      return NextResponse.redirect(new URL('/auth?error=No ID token returned', request.url));
    }

    // Decode ID token to get user info (email)
    const decoded = jwt.decode(idToken) as { email?: string; sub?: string } | null;
    if (!decoded || !decoded.email) {
      return NextResponse.redirect(new URL('/auth?error=Invalid ID token', request.url));
    }

    // [ Look up or create the user in your database here ]
    // e.g. await db.user.upsert({ where: { email: decoded.email }, create: { email: decoded.email } });

    // Issue a JWT cookie representing the session
    const token = jwt.sign({ email: decoded.email, sub: decoded.sub }, JWT_SECRET, { expiresIn: '7d' });
    cookieStore.set('auth_token', token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
      maxAge: 7 * 24 * 60 * 60, // 7 days
    });

    // Redirect to /home
    return NextResponse.redirect(new URL('/home', request.url));
  } catch (error) {
    console.error('Google OAuth Error:', error);
    return NextResponse.redirect(new URL('/auth?error=Internal server error', request.url));
  }
}
