import type { NextApiRequest, NextApiResponse } from 'next';
import { authorizationControlClient, authorizationDataClient } from '../../lib/grpc/node-client';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    // Verificar conexión gRPC y Database REALMENTE (NO mocks, NO stubs)
    let grpcServerStatus = 'disconnected';
    let databaseStatus = 'disconnected';
    let databaseDetails: any = null;

    // 1. Verificar gRPC server (Control Plane) - REAL CALL
    try {
      console.log('🔍 REAL gRPC call: listPolicyStores()');
      await new Promise((resolve, reject) => {
        authorizationControlClient.listPolicyStores({ max_results: 1 }, (error: any, response: any) => {
          if (error) {
            console.error('❌ gRPC Control Plane ERROR:', error);
            reject(error);
          } else {
            console.log('✅ gRPC Control Plane SUCCESS:', response);
            resolve(true);
          }
        });
      });
      grpcServerStatus = 'connected';
    } catch (grpcError: any) {
      console.error('❌ gRPC server (control plane) FAILED:', grpcError.code || grpcError.message);
      grpcServerStatus = 'disconnected';
    }

    // 2. Verificar database (Data Plane) - REAL CALL
    try {
      console.log('🔍 REAL gRPC call: isAuthorized() with real DB query');
      await new Promise((resolve, reject) => {
        // Crear request REAL (no dummy) para probar DB real
        const realRequest = {
          policy_store_id: 'health-check-real-db-test',
          principal: { entity_type: 'User', entity_id: 'real-health-check' },
          action: { entity_type: 'Action', entity_id: 'real-health-check-action' },
          resource: { entity_type: 'Resource', entity_id: 'real-health-check-resource' },
          context: '{}',
          entities: []
        };

        authorizationDataClient.isAuthorized(realRequest, (error: any, response: any) => {
          if (error) {
            console.log('⚠️ isAuthorized returned:', error.code, error.message);
            // Códigos válidos: NOT_FOUND (política no existe), OK (política existe)
            if (error.code === 5 || error.code === undefined) {
              // NOT_FOUND significa que la DB está funcionando pero la política no existe
              console.log('✅ Database is CONNECTED and RESPONDING (policy not found is OK)');
              databaseStatus = 'connected';
              databaseDetails = { connected: true, errorCode: error.code, message: error.message };
              resolve(true);
            } else {
              console.error('❌ Database connectivity ERROR:', error.code, error.message);
              databaseStatus = 'disconnected';
              databaseDetails = { connected: false, errorCode: error.code, message: error.message };
              reject(error);
            }
          } else {
            console.log('✅ Database is CONNECTED - Authorization response:', response);
            databaseStatus = 'connected';
            databaseDetails = { connected: true, response: response };
            resolve(true);
          }
        });
      });
    } catch (dbError: any) {
      console.error('❌ Database (data plane) FAILED:', dbError.code || dbError.message);
      databaseStatus = 'disconnected';
      databaseDetails = { connected: false, error: dbError.message };
    }

    // Determinar estado general basado en verificaciones REALES
    const overallStatus = (grpcServerStatus === 'connected' && databaseStatus === 'connected')
      ? 'healthy'
      : 'degraded';

    const response = {
      status: overallStatus,
      grpc_server: grpcServerStatus,
      database: databaseStatus,
      timestamp: new Date().toISOString(),
      verification_type: 'REAL_GRPC_CALLS_NO_MOCKS',
      checks: {
        control_plane: {
          status: grpcServerStatus,
          method: 'authorizationControlClient.listPolicyStores',
          type: 'REAL_GRPC_CALL'
        },
        data_plane: {
          status: databaseStatus,
          method: 'authorizationDataClient.isAuthorized',
          type: 'REAL_GRPC_CALL',
          details: databaseDetails
        }
      },
      message: overallStatus === 'healthy'
        ? 'All systems operational (verified with REAL gRPC calls)'
        : 'Some systems are not responding (verified with REAL gRPC calls)'
    };

    console.log('🏥 Health Check Result:', JSON.stringify(response, null, 2));
    res.status(200).json(response);

  } catch (error: any) {
    console.error('💥 Health check FATAL ERROR:', error);
    res.status(500).json({
      status: 'unhealthy',
      error: 'Internal server error',
      details: error.message,
      timestamp: new Date().toISOString(),
      verification_type: 'REAL_GRPC_CALLS_NO_MOCKS'
    });
  }
}
