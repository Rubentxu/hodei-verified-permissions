import { NextResponse } from 'next/server';
import { authorizationControlClient } from '@/lib/grpc/node-client';

// GET /api/policy-stores/[id]/policies/[policyId]
export async function GET(request: Request, { params }: { params: { id: string, policyId: string } }) {
  try {
    const { id, policyId } = params;
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.getPolicy({ policy_store_id: id, policy_id: policyId }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('GetPolicy error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}

// PUT /api/policy-stores/[id]/policies/[policyId]
export async function PUT(request: Request, { params }: { params: { id: string, policyId: string } }) {
  try {
    const { id, policyId } = params;
    const { policy } = await request.json();

    if (!policy) {
      return NextResponse.json({ error: 'Policy content is required' }, { status: 400 });
    }

    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.updatePolicy({ policy_store_id: id, policy_id: policyId, policy }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('UpdatePolicy error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}

// DELETE /api/policy-stores/[id]/policies/[policyId]
export async function DELETE(request: Request, { params }: { params: { id: string, policyId: string } }) {
  try {
    const { id, policyId } = params;
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.deletePolicy({ policy_store_id: id, policy_id: policyId }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('DeletePolicy error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}
