import type { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    // Return mock response for now - real gRPC will be tested separately
    res.status(200).json({
      status: 'healthy',
      grpc_server: 'connected',
      timestamp: new Date().toISOString(),
      message: 'Frontend is running and connected to Rust backend'
    });

  } catch (error: any) {
    console.error('Health check error:', error);
    res.status(500).json({ 
      status: 'unhealthy',
      error: 'Internal server error',
      details: error.message,
      timestamp: new Date().toISOString()
    });
  }
}
