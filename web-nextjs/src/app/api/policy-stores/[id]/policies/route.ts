import { NextResponse } from 'next/server';
import { authorizationControlClient } from '@/lib/grpc/node-client';

// GET /api/policy-stores/[id]/policies
export async function GET(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.listPolicies({ policy_store_id: id }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('ListPolicies error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}

// POST /api/policy-stores/[id]/policies
export async function POST(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const { policy_id, policy } = await request.json();

    if (!policy_id || !policy) {
      return NextResponse.json({ error: 'policy_id and policy are required' }, { status: 400 });
    }

    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.createPolicy({ policy_store_id: id, policy_id, policy }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('CreatePolicy error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}
