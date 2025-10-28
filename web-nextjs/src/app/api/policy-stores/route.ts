import { NextResponse } from 'next/server';
import { authorizationControlClient } from '@/lib/grpc/node-client';

export async function GET() {
  try {
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.listPolicyStores({}, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('ListPolicyStores error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}

export async function POST(request: Request) {
  try {
    const { description } = await request.json();
    const response = await new Promise((resolve, reject) => {
      authorizationControlClient.createPolicyStore({ description }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('CreatePolicyStore error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}
