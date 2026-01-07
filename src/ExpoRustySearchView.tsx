import { requireNativeView } from 'expo';
import * as React from 'react';

import { ExpoRustySearchViewProps } from './ExpoRustySearch.types';

const NativeView: React.ComponentType<ExpoRustySearchViewProps> =
  requireNativeView('ExpoRustySearch');

export default function ExpoRustySearchView(props: ExpoRustySearchViewProps) {
  return <NativeView {...props} />;
}
