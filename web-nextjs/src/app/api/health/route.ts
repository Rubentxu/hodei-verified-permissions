import { NextResponse } from 'next/server';
import { healthClient } from '@/lib/grpc/node-client';

export async function GET() {
  try {
    const response = await new Promise((resolve, reject) => {
      healthClient.check({ service: '' }, (error: any, res: any) => {
        if (error) {
          return reject(error);
        }
        resolve(res);
      });
    });
    return NextResponse.json(response);
  } catch (error: any) {
    console.error('Health check error:', error);
    return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
  }
}
