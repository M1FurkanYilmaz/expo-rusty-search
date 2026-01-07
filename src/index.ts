// Reexport the native module. On web, it will be resolved to ExpoRustySearchModule.web.ts
// and on native platforms to ExpoRustySearchModule.ts
export { default } from './ExpoRustySearchModule';
export { default as ExpoRustySearchView } from './ExpoRustySearchView';
export * from  './ExpoRustySearch.types';
