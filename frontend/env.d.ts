/// <reference types="@rsbuild/core/types" />

declare module "*.vue" 
{
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}


interface ImportMetaEnv 
{
  // import.meta.env.PUBLIC_FOO
  readonly PUBLIC_FOO: string;
}

interface ImportMeta 
{
  readonly env: ImportMetaEnv;
}