import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import path from 'path';

const PROTO_PATH = path.join(process.cwd(), '../../proto/authorization.proto');

const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
});

const proto = grpc.loadPackageDefinition(packageDefinition) as any;

const VERIFIED_PERMISSIONS_ADDR = process.env.VERIFIED_PERMISSIONS_ADDR || 'localhost:50051';

export const authorizationControlClient = new proto.authorization.AuthorizationControl(
  VERIFIED_PERMISSIONS_ADDR,
  grpc.credentials.createInsecure()
);

export const authorizationDataClient = new proto.authorization.AuthorizationData(
  VERIFIED_PERMISSIONS_ADDR,
  grpc.credentials.createInsecure()
);

export const healthClient = new proto.grpc.health.v1.Health(
    VERIFIED_PERMISSIONS_ADDR,
    grpc.credentials.createInsecure()
);
