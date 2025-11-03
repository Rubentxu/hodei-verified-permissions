/** @type {import('next').NextConfig} */
const nextConfig = {
  typescript: {
    ignoreBuildErrors: true, // Disable type checking during build to save memory
  },
  productionBrowserSourceMaps: false, // Disable source maps to save memory
  experimental: {
    webpackMemoryOptimizations: true, // Enable webpack memory optimizations
    webpackBuildWorker: true, // Use build worker to reduce memory pressure
    preloadEntriesOnStart: false, // Don't preload all entries on start
    staticPageGenerationTimeout: 120, // Increase timeout for slow pages
  },
  webpack: (config, { dev }) => {
    // Disable webpack's in-memory cache during production builds
    if (config.cache && !dev) {
      config.cache = Object.freeze({ type: 'memory' });
    }
    return config;
  },
}

module.exports = nextConfig
