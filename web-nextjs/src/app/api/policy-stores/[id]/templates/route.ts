import { NextResponse } from 'next/server';
import { authorizationControlClient } from '@/lib/grpc/node-client';

// GET /api/policy-stores/[id]/templates
export async function GET(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.listPolicyTemplates({ policy_store_id: id }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('ListPolicyTemplates error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}

// POST /api/policy-stores/[id]/templates
export async function POST(request: Request, { params }: { params: { id: string } }) {
  try {
    const { id } = params;
    const { policy_template_id, policy_src } = await request.json();

    if (!policy_template_id || !policy_src) {
      return NextResponse.json({ error: 'policy_template_id and policy_src are required' }, { status: 400 });
    }

    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.createPolicyTemplate({ policy_store_id: id, policy_template_id, policy_src }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('CreatePolicyTemplate error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}
