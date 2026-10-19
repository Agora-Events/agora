import { NextResponse } from 'next/server';
import jwt from 'jsonwebtoken';
import { cookies } from 'next/headers';

const JWT_SECRET = process.env.JWT_SECRET || 'fallback_secret_for_dev';

export async function GET(request: Request) {
  try {
    const { searchParams } = new URL(request.url);
    const code = searchParams.get('code');

    if (!code) {
      return NextResponse.redirect(new URL('/auth?error=Missing code', request.url));
    }

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
    console.error('Google OAuth Error:', error);
    return NextResponse.redirect(new URL('/auth?error=Internal server error', request.url));
  }
}
