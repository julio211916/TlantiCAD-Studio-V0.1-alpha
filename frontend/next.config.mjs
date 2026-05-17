import { fileURLToPath, URL } from 'node:url';

const frontendRoot = fileURLToPath(new URL('.', import.meta.url));
const cadWorkspaceRoot = fileURLToPath(new URL('./workspaces/tlanti-cad', import.meta.url));

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  distDir: '.next',
  images: {
    unoptimized: true,
  },
  outputFileTracingRoot: fileURLToPath(new URL('..', import.meta.url)),
  typescript: {
    ignoreBuildErrors: false,
  },
  webpack(config, { dev }) {
    config.resolve.alias = {
      ...config.resolve.alias,
      '@/components/tlantidb': `${frontendRoot}/workspaces/tlanti-db`,
      '@/components/asset-library': `${cadWorkspaceRoot}/asset-library`,
      '@/components/workspaces': `${frontendRoot}/workspaces`,
      '@/components/cad': cadWorkspaceRoot,
      '@/components/ui': `${frontendRoot}/ui`,
      '@tlanticad/ui': `${frontendRoot}/ui/tlanticad-ui.tsx`,
      '@/features': `${cadWorkspaceRoot}/features`,
      '@/providers': `${frontendRoot}/providers`,
      '@/platform': `${frontendRoot}/platform`,
      '@/hooks': `${frontendRoot}/hooks`,
      '@/stores': `${frontendRoot}/stores`,
      '@/core': `${frontendRoot}/core`,
      '@/types': `${frontendRoot}/types`,
      '@/utils': `${frontendRoot}/utils`,
      '@/lib': `${frontendRoot}/lib`,
      '@': frontendRoot,
    };
    config.resolve.fallback = {
      ...(config.resolve.fallback ?? {}),
      fs: false,
      path: false,
      crypto: false,
    };
    if (!dev && Array.isArray(config.optimization?.minimizer)) {
      config.optimization.minimizer = config.optimization.minimizer.filter((minimizer) => {
        const name = minimizer?.constructor?.name ?? '';
        const source = String(minimizer);
        return !/Css.*Minimizer|MinifyCss|CssMinimizerPlugin/i.test(`${name} ${source}`);
      });
    }
    return config;
  },
};

export default nextConfig;
