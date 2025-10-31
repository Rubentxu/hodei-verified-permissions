import { NextResponse } from 'next/server';
import * as grpc from '@grpc/grpc-js';

export function handleGRPCError(error: any) {
    const status = error.code || grpc.status.INTERNAL;
    const message = error.details || 'An unexpected error occurred';

    let statusCode = 500;
    switch (status) {
        case grpc.status.NOT_FOUND:
            statusCode = 404;
            break;
        case grpc.status.INVALID_ARGUMENT:
            statusCode = 400;
            break;
        case grpc.status.PERMISSION_DENIED:
            statusCode = 403;
            break;
        case grpc.status.UNAUTHENTICATED:
            statusCode = 401;
            break;
        case grpc.status.ALREADY_EXISTS:
            statusCode = 409;
            break;
    }

    return new NextResponse(JSON.stringify({ message }), {
        status: statusCode,
        headers: { 'Content-Type': 'application/json' },
    });
}
