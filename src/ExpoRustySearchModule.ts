import { NativeModule, requireNativeModule } from "expo";
import { ExpoRustySearchModuleEvents } from "./ExpoRustySearch.types";

declare class ExpoRustySearchModule extends NativeModule<ExpoRustySearchModuleEvents> {
  // Your New Rust Functions
  initializeIndex(indexPath: string): Promise<string>;
  addDocument(title: string, body: string): Promise<string>;
  search(query: string): Promise<string>;
  getDocumentCount(): Promise<string>;

  // Existing methods
  PI: number;
  hello(): string;
  setValueAsync(value: string): Promise<void>;
}

const module = requireNativeModule<ExpoRustySearchModule>("ExpoRustySearch");
export default module;

// High-level wrapper for cleaner usage
export async function searchIndex(query: string): Promise<any[]> {
  const result = await module.search(query);
  return JSON.parse(result); // Assuming Rust returns a JSON string
}
