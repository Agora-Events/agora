import { NextResponse } from 'next/server';
import jwt from 'jsonwebtoken';
import { cookies } from 'next/headers';
import crypto from 'crypto';

const JWT_SECRET = process.env.JWT_SECRET || 'fallback_secret_for_dev';

// Name of the short-lived cookie that carries the anti-CSRF `state` value
// between the redirect to Apple and the callback below.
const APPLE_STATE_COOKIE = 'apple_oauth_state';
const STATE_COOKIE_MAX_AGE = 10 * 60; // 10 minutes — just long enough for sign-in

// Constant-time string comparison so a mismatched `state` can't be detected
// via response-time differences.
function timingSafeEqual(a: string, b: string): boolean {
  const bufA = Buffer.from(a);
  const bufB = Buffer.from(b);
  if (bufA.length !== bufB.length) return false;
  return crypto.timingSafeEqual(bufA, bufB);
}

// Starts the OAuth redirect: generates a random `state`, stashes it in a
// short-lived httpOnly cookie, and sends the browser to Apple with that same
// value so the callback below can confirm the request round-tripped through
// this browser (login CSRF protection).
async function startAppleOAuth() {
  const cookieStore = await cookies();
  const generatedState = crypto.randomBytes(32).toString('hex');
  cookieStore.set(APPLE_STATE_COOKIE, generatedState, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    path: '/',
    maxAge: STATE_COOKIE_MAX_AGE,
  });

  const authorizeUrl = new URL('https://appleid.apple.com/auth/authorize');
  authorizeUrl.searchParams.set('client_id', process.env.APPLE_CLIENT_ID || '');
  authorizeUrl.searchParams.set(
    'redirect_uri',
    process.env.APPLE_REDIRECT_URI || 'http://localhost:3000/api/auth/apple'
  );
  authorizeUrl.searchParams.set('response_type', 'code');
  authorizeUrl.searchParams.set('scope', 'name email');
  authorizeUrl.searchParams.set('response_mode', 'form_post');
  authorizeUrl.searchParams.set('state', generatedState);

  return NextResponse.redirect(authorizeUrl);
}

// Validates the returned `state` against the cookie set by startAppleOAuth,
// clearing the cookie either way so it can't be replayed.
async function verifyState(state: string | null): Promise<boolean> {
  const cookieStore = await cookies();
  const storedState = cookieStore.get(APPLE_STATE_COOKIE)?.value;
  cookieStore.delete(APPLE_STATE_COOKIE);
  return Boolean(state && storedState && timingSafeEqual(state, storedState));
}

async function handleAppleOAuth(code: string, state: string | null, request: Request) {
  const stateValid = await verifyState(state);
  if (!stateValid) {
    return NextResponse.redirect(new URL('/auth?error=Invalid state parameter', request.url));
  }

  try {
    // Exchange auth code for ID token via Apple APIs
    const tokenResponse = await fetch('https://appleid.apple.com/auth/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        client_id: process.env.APPLE_CLIENT_ID || '',
        client_secret: process.env.APPLE_CLIENT_SECRET || '', 
        code,
        grant_type: 'authorization_code',
        redirect_uri: process.env.APPLE_REDIRECT_URI || 'http://localhost:3000/api/auth/apple',
      }),
    });

    if (!tokenResponse.ok) {
      return NextResponse.redirect(new URL('/auth?error=Apple OAuth failed', request.url));
    }

    const tokenData = await tokenResponse.json();
    const idToken = tokenData.id_token;

    if (!idToken) {
      return NextResponse.redirect(new URL('/auth?error=No ID token returned', request.url));
    }

    // Decode ID token to get user info (email)
    const decoded = jwt.decode(idToken) as { email?: string; sub?: string } | null;
    
    // Apple ID tokens may not contain email depending on scope initially authorized,
    // but typically we can fall back to the subject string if explicitly allowed, 
    // or just rely on decoded.email
    if (!decoded || (!decoded.email && !decoded.sub)) {
      return NextResponse.redirect(new URL('/auth?error=Invalid ID token', request.url));
    }
    
    const userEmail = decoded.email || `${decoded.sub}@apple.oauth.stub`;

    // [ Look up or create the user in your database here ]
    // e.g. await db.user.upsert({ where: { email: userEmail }, create: { email: userEmail } });

    // Issue a JWT cookie representing the session
    const token = jwt.sign({ email: userEmail, sub: decoded.sub }, JWT_SECRET, { expiresIn: '7d' });
    const cookieStore = await cookies();
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
    console.error('Apple OAuth Error:', error);
    return NextResponse.redirect(new URL('/auth?error=Internal server error', request.url));
  }
}

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const code = searchParams.get('code');
  const state = searchParams.get('state');

  // No `code` yet: this is the request that starts the OAuth redirect.
  if (!code) {
    return startAppleOAuth();
  }

  return handleAppleOAuth(code, state, request);
}

// Apple uses POST with form_post response_mode when scopes like name and email are requested
export async function POST(request: Request) {
  try {
    const formData = await request.formData();
    const code = formData.get('code');
    const state = formData.get('state');

    if (!code || typeof code !== 'string') {
      return NextResponse.redirect(new URL('/auth?error=Missing code', request.url));
    }

    return handleAppleOAuth(code, typeof state === 'string' ? state : null, request);
  } catch {
    return NextResponse.redirect(new URL('/auth?error=Invalid POST request', request.url));
  }
}
