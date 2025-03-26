import { defineConfig } from '@rsbuild/core';
import { pluginVue } from '@rsbuild/plugin-vue';
import { pluginPug } from '@rsbuild/plugin-pug';
import { pluginSass } from '@rsbuild/plugin-sass';
export default defineConfig({
    plugins: [pluginVue(), pluginPug(), pluginSass()],
    resolve: {
        //alias: {
            // '@/*': './src/*',
            // '@styles/': './src/styles',
            // '@types/': './src/@types',
            // '@images/': './src/assets/images',
            // '@components/': './src/components',
            // '@services/': './src/services',
            //'@svg': './assets/svg',
          //}
    },
    source: {
        entry: {
            index: "./src/main.ts"
        },
        
    },
    output: {
        distPath: {
			root: "dist",
			assets: "static/assets",
			font: "static/fonts",
			image: "static/images",
			svg: "static/images",
			media: "static/media",
			js: "js",
			css: "styles",
			jsAsync: "js",
			cssAsync: "styles",
			wasm: "wasm"
		},
    },
    server: {
        port: 8080,
        strictPort: true,
    },
    tools: {
        rspack: {
            watchOptions: {
                ignored: ['**/node_modules/**'],
            }
        }
    },
    html: {
        template: 'index.html',
        // appIcon:{
        //     icons: [
        //         { src: './assets/svg/bell.svg', size: 36}
        //     ]
        // },
        //favicon: './assets/favicon.ico',
        meta: {
          charset: 'UTF-8',
          viewport: 'width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=0, minimum-scale=1.0, viewport-fit=cover'
        },
        title: 'Воу воу'
      },
});