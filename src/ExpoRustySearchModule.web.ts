import { registerWebModule, NativeModule } from 'expo';

import { ExpoRustySearchModuleEvents } from './ExpoRustySearch.types';

class ExpoRustySearchModule extends NativeModule<ExpoRustySearchModuleEvents> {
  PI = Math.PI;
  async setValueAsync(value: string): Promise<void> {
    this.emit('onChange', { value });
  }
  hello() {
    return 'Hello world! 👋';
  }
}

export default registerWebModule(ExpoRustySearchModule, 'ExpoRustySearchModule');
