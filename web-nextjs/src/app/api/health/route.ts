import { NextRequest, NextResponse } from 'next/server';
import { healthCheck } from '@/lib/grpc/node-client';

export async function GET() {
  console.log('[API health] start');
  try {
    const ok = await healthCheck();
    console.log('[API health] result:', ok);
    return NextResponse.json({ status: ok ? 'Connected' : 'Error' }, { status: ok ? 200 : 503 });
  } catch (e: any) {
    console.error('[API health] error', e);
    return NextResponse.json({ status: 'Error', error: e.message }, { status: 503 });
  }
}