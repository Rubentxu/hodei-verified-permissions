import { Html, Head, Main, NextScript } from 'next/document';

export default function Document() {
  return (
    <Html lang="en">
      <Head>
        <title>Hodei Verified Permissions</title>
        <meta name="description" content="Backend for Frontend with gRPC connectivity to Rust server" />
      </Head>
      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
