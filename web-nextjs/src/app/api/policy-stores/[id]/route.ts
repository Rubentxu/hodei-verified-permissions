import { NextRequest, NextResponse } from 'next/server';
import { getPolicyStore, deletePolicyStore } from '@/lib/grpc/node-client';

export async function GET(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const { id } = await params;
    const store = await getPolicyStore(id);
    return NextResponse.json(store);
  } catch (e: any) {
    console.error('[API policy-stores [id] GET] error', e);
    return NextResponse.json({ error: e.message }, { status: 404 });
  }
}

export async function DELETE(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const { id } = await params;
    await deletePolicyStore(id);
    return NextResponse.json({}, { status: 204 });
  } catch (e: any) {
    console.error('[API policy-stores [id] DELETE] error', e);
    return NextResponse.json({ error: e.message }, { status: 500 });
  }
}