import { NextResponse } from 'next/server';
import jwt from 'jsonwebtoken';
import { cookies } from 'next/headers';

const JWT_SECRET = process.env.JWT_SECRET || 'fallback_secret_for_dev';

async function handleAppleOAuth(code: string, request: Request) {
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

  if (!code) {
    return NextResponse.redirect(new URL('/auth?error=Missing code', request.url));
  }

  return handleAppleOAuth(code, request);
}

// Apple uses POST with form_post response_mode when scopes like name and email are requested 
export async function POST(request: Request) {
  try {
    const formData = await request.formData();
    const code = formData.get('code');

    if (!code || typeof code !== 'string') {
      return NextResponse.redirect(new URL('/auth?error=Missing code', request.url));
    }

    return handleAppleOAuth(code, request);
  } catch {
    return NextResponse.redirect(new URL('/auth?error=Invalid POST request', request.url));
  }
}
