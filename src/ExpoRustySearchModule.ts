import { NativeModule, requireNativeModule } from 'expo';

import { ExpoRustySearchModuleEvents } from './ExpoRustySearch.types';

declare class ExpoRustySearchModule extends NativeModule<ExpoRustySearchModuleEvents> {
  PI: number;
  hello(): string;
  setValueAsync(value: string): Promise<void>;
}

// This call loads the native module object from the JSI.
export default requireNativeModule<ExpoRustySearchModule>('ExpoRustySearch');
