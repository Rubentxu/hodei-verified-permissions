import { NextResponse } from 'next/server';
import { authorizationControlClient } from '@/lib/grpc/node-client';

// POST /api/policy-stores/[id]/is_authorized
export async function POST(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const { principal, action, resource, context } = await request.json();

    if (!principal || !action || !resource) {
      return NextResponse.json({ error: 'principal, action, and resource are required' }, { status: 400 });
    }

    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.isAuthorized({ 
        policy_store_id: id, 
        principal, 
        action, 
        resource, 
        context 
      }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('IsAuthorized error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}
