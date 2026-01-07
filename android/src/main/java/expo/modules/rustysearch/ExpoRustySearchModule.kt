package expo.modules.rustysearch

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import java.net.URL

class ExpoRustySearchModule : Module() {
  private external fun initializeIndex(indexPath: String): String
  private external fun addDocument(title: String, body: String): String
  private external fun searchBasic(query: String): String
  private external fun getDocumentCount(): String

  companion object {
    init {
      // Use your actual crate name from Cargo.toml here
      System.loadLibrary("expo_search_core") 
    }
  }

  override fun definition() = ModuleDefinition {
    Name("ExpoRustySearch")

    Constant("PI") {
      Math.PI
    }

    // Defines event names that the module can send to JavaScript.
    Events("onChange")

    // Defines a JavaScript synchronous function that runs the native code on the JavaScript thread.
    Function("hello") {
      "Hello world! 👋"
    }

    // Defines a JavaScript function that always returns a Promise and whose native code
    // is by default dispatched on the different thread than the JavaScript runtime runs on.
    AsyncFunction("setValueAsync") { value: String ->
      // Send an event to JavaScript.
      sendEvent("onChange", mapOf(
        "value" to value
      ))
    }

    AsyncFunction("initializeIndex") { indexPath: String ->
      initializeIndex(indexPath)
    }

    AsyncFunction("addDocument") { title: String, body: String ->
      addDocument(title, body)
    }

    AsyncFunction("search") { query: String ->
      searchBasic(query)
    }

    AsyncFunction("getDocumentCount") {
      getDocumentCount()
    }

    // Enables the module to be used as a native view. Definition components that are accepted as part of
    // the view definition: Prop, Events.
    View(ExpoRustySearchView::class) {
      // Defines a setter for the `url` prop.
      Prop("url") { view: ExpoRustySearchView, url: URL ->
        view.webView.loadUrl(url.toString())
      }
      // Defines an event that the view can send to JavaScript.
      Events("onLoad")
    }
  }
}
