import { NativeModule, requireNativeModule } from "expo";
import type { StyleProp, ViewStyle } from "react-native";

export type OnLoadEventPayload = {
  url: string;
};

export type ExpoRustySearchModuleEvents = {
  onChange: (params: ChangeEventPayload) => void;
};

export type ChangeEventPayload = {
  value: string;
};

export type ExpoRustySearchViewProps = {
  url: string;
  onLoad: (event: { nativeEvent: OnLoadEventPayload }) => void;
  style?: StyleProp<ViewStyle>;
};

export type SearchResult = {
  title: string;
  body: string;
  score: number;
};

export type DocumentType = {
  title: string;
  body: string;
};

declare class ExpoRustySearchModule extends NativeModule<ExpoRustySearchModuleEvents> {
  // Rust search functions
  initializeIndex(): Promise<string>;
  clearIndex(): Promise<string>;
  addDocument(title: string, body: string): Promise<string>;
  addDocumentsBulk(json: string): Promise<string>;
  searchDocuments(query: string): Promise<string>;
  getDocumentCount(): Promise<string>;

  // Existing methods
  PI: number;
  hello(): string;
  setValueAsync(value: string): Promise<void>;
}

const ExpoRustySearch =
  requireNativeModule<ExpoRustySearchModule>("ExpoRustySearch");

export const {
  initializeIndex,
  clearIndex,
  addDocument,
  searchDocuments,
  addDocumentsBulk,
  getDocumentCount,
} = ExpoRustySearch;
