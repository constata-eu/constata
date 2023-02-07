const webpack = require('webpack');
module.exports = {
  webpack: {
    configure: config => {
      const wasmExtensionRegExp = /\.wasm$/;
      config.resolve.extensions.push('.wasm');
      config.experiments = {
        asyncWebAssembly: true,
      //  asyncWebAssembly: false,
      //  lazyCompilation: true,
        syncWebAssembly: true,
        topLevelAwait: true,
      };
      config.resolve.fallback = {
        buffer: require.resolve('buffer/'),
        stream: require.resolve('stream-browserify/')
      }
      config.module.rules.forEach((rule) => {
        (rule.oneOf || []).forEach((oneOf) => {
          if (oneOf.type === "asset/resource") {
            oneOf.exclude.push(wasmExtensionRegExp);
          }
        });
      });
      config.plugins.push(new webpack.ProvidePlugin({
        Buffer: ['buffer', 'Buffer'],
      }));

      return config;
    },
  },
}
