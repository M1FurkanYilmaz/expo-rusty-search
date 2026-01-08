package expo.modules.rustysearch

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

class ExpoRustySearchModule : Module() {

  // JNI functions return String directly
  private external fun initializeIndex(): String
  private external fun clearIndex(): String
  private external fun addDocument(title: String, body: String): String
  private external fun addDocumentsBulk(json: String): String
  private external fun searchDocuments(query: String): String
  private external fun getDocumentCount(): String

  companion object {
    init {
      System.loadLibrary("expo_search_core")
    }
  }

  override fun definition() = ModuleDefinition {
    Name("ExpoRustySearch")

    Events("onChange")

    Function("hello") {
      "Hello world!"
    }

    AsyncFunction("initializeIndex") {
      initializeIndex()
    }

    AsyncFunction("clearIndex") {
      clearIndex()
    }

    AsyncFunction("addDocument") { title: String, body: String ->
      addDocument(title, body)
    }

    AsyncFunction("addDocumentsBulk") { json: String ->
      addDocumentsBulk(json)
    }

    AsyncFunction("searchDocuments") { query: String ->
      searchDocuments(query)
    }

    AsyncFunction("getDocumentCount") {
      getDocumentCount()
    }
  }
}
