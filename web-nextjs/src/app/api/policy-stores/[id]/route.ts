import { NextResponse } from 'next/server';
import { authorizationControlClient } from '@/lib/grpc/node-client';

export async function GET(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.getPolicyStore({ policy_store_id: id }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('GetPolicyStore error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}

export async function DELETE(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.deletePolicyStore({ policy_store_id: id }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('DeletePolicyStore error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}
