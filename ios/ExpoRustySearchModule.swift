import ExpoModulesCore

public class ExpoRustySearchModule: Module {
  // Helper to handle Rust string pointers safely
  func rustToString(_ ptr: UnsafePointer<Int8>?) -> String {
    guard let ptr = ptr else { return "" }
    let result = String(cString: ptr)
    free_string(UnsafeMutablePointer(mutating: ptr))
    return result
  }

  public func definition() -> ModuleDefinition {
    Name("ExpoRustySearch")

    Constant("PI") {
      Double.pi
    }

    Events("onChange")

    Function("hello") {
      return "Hello world! 👋"
    }

    AsyncFunction("setValueAsync") { (value: String) in
      self.sendEvent("onChange", [
        "value": value
      ])
    }

    AsyncFunction("initializeIndex") { () -> String in
      return self.rustToString(initialize_index())
    }

    AsyncFunction("clearIndex") { () -> String in
      return self.rustToString(clear_index())
    }

    AsyncFunction("addDocument") { (title: String, body: String) -> String in
      return self.rustToString(add_document(title, body))
    }

    AsyncFunction("addDocumentsBulk") { (json: String) -> String in
      return self.rustToString(add_documents_bulk(json))
    }

    AsyncFunction("searchDocuments") { (query: String) -> String in
      return self.rustToString(search_documents(query))
    }

    AsyncFunction("getDocumentCount") { () -> String in
      return self.rustToString(get_document_count())
    }
  }
}