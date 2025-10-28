import { NextRequest, NextResponse } from 'next/server';
import { listPolicyStores, createPolicyStore } from '@/lib/grpc/node-client';

export async function GET() {
  try {
    const stores = await listPolicyStores();
    return NextResponse.json(stores);
  } catch (e: any) {
    console.error('[API policy-stores GET] error', e);
    return NextResponse.json({ error: e.message }, { status: 500 });
  }
}

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();
    const result = await createPolicyStore(body.description);
    return NextResponse.json(result, { status: 201 });
  } catch (e: any) {
    console.error('[API policy-stores POST] error', e);
    return NextResponse.json({ error: e.message }, { status: 500 });
  }
}