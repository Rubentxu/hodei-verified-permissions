import { NextResponse } from 'next/server';
import { authorizationControlClient } from '@/lib/grpc/node-client';

// GET /api/policy-stores/[id]/schema
export async function GET(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.getSchema({ policy_store_id: id }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('GetSchema error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}

// PUT /api/policy-stores/[id]/schema
export async function PUT(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const { schema } = await request.json();

    if (!schema) {
      return NextResponse.json({ error: 'Schema is required' }, { status: 400 });
    }

    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.putSchema({ policy_store_id: id, schema }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('PutSchema error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}
