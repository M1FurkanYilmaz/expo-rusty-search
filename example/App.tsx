import React, { useState, useEffect } from "react";
import {
  View,
  Text,
  TextInput,
  Button,
  FlatList,
  StyleSheet,
  Alert,
  ScrollView,
} from "react-native";
import AsyncStorage from "@react-native-async-storage/async-storage";
// 1. UPDATE IMPORTS: Added addDocumentsBulk and getDocumentCount
import {
  SearchResult,
  addDocument,
  addDocumentsBulk,
  getDocumentCount,
  initializeIndex,
  searchDocuments,
  DocumentType,
} from "expo-rusty-search";

const STORAGE_KEY = "@search_documents";

export default function App() {
  const [results, setResults] = useState<SearchResult[]>([]);
  const [count, setCount] = useState(0);
  const [initialized, setInitialized] = useState(false);
  const [initMessage, setInitMessage] = useState("Initializing...");
  const [searchQuery, setSearchQuery] = useState("");

  // Helper to fetch count safely and parse string to number
  const fetchCount = async () => {
    try {
      const countStr = await getDocumentCount();
      setCount(parseInt(countStr, 10) || 0);
    } catch (e) {
      console.error("Error fetching count", e);
    }
  };

  // Load documents from AsyncStorage and index them
  useEffect(() => {
    const initIndex = async () => {
      try {
        // Initialize the in-memory index
        const result = await initializeIndex();
        console.log("Index initialized:", result);
        setInitMessage(result);

        // Load saved documents from AsyncStorage
        const savedDocsJson = await AsyncStorage.getItem(STORAGE_KEY);
        if (savedDocsJson) {
          const savedDocs: Document[] = JSON.parse(savedDocsJson);
          console.log(`Loading ${savedDocs.length} documents from storage...`);

          // 2. FIXED: Use addDocumentsBulk with JSON.stringify
          // Your Rust/Swift code expects a JSON string, not an object array
          const bulkResult = await addDocumentsBulk(JSON.stringify(savedDocs));
          console.log("Bulk add result:", bulkResult);
        }

        // 3. FIXED: Fetch count using native module
        await fetchCount();

        setInitialized(true);
      } catch (error) {
        console.error("Failed to initialize index:", error);
        setInitMessage(`Error: ${error}`);
        Alert.alert(
          "Error",
          "Failed to initialize search index: " + String(error)
        );
      }
    };

    initIndex();
  }, []);

  // Save documents to AsyncStorage
  const saveDocumentsToStorage = async (docs: DocumentType[]) => {
    try {
      await AsyncStorage.setItem(STORAGE_KEY, JSON.stringify(docs));
    } catch (error) {
      console.error("Failed to save documents:", error);
    }
  };

  // Get all documents from AsyncStorage
  const getDocumentsFromStorage = async (): Promise<DocumentType[]> => {
    try {
      const savedDocsJson = await AsyncStorage.getItem(STORAGE_KEY);
      if (savedDocsJson) {
        return JSON.parse(savedDocsJson);
      }
    } catch (error) {
      console.error("Failed to load documents:", error);
    }
    return [];
  };

  const onAddDocument = async () => {
    if (!initialized) {
      Alert.alert("Error", "Index not initialized yet");
      return;
    }

    try {
      const newDoc: DocumentType = {
        title: `Test Document ${count + 1}`,
        body: "This is a test document sent to Rust. It contains searchable content about programming and testing.",
      };

      // Add to Rust index
      const result = await addDocument(newDoc.title, newDoc.body);
      console.log("Add result:", result);

      // Save to AsyncStorage
      const existingDocs = await getDocumentsFromStorage();
      existingDocs.push(newDoc);
      await saveDocumentsToStorage(existingDocs);

      // Update count
      await fetchCount();

      Alert.alert("Success", `Document added!`);
    } catch (error) {
      console.error("Add error:", error);
      Alert.alert("Error", String(error));
    }
  };

  const onAddMultipleDocuments = async () => {
    if (!initialized) {
      Alert.alert("Error", "Index not initialized yet");
      return;
    }

    try {
      const newDocs: DocumentType[] = [
        {
          title: "Introduction to Rust",
          body: "Rust is a systems programming language focused on safety and performance.",
        },
        {
          title: "React Native Development",
          body: "React Native allows you to build mobile apps using JavaScript and React.",
        },
        {
          title: "Search Engines",
          body: "Full-text search engines use inverted indices to quickly find documents.",
        },
      ];

      // 4. FIXED: Use addDocumentsBulk with JSON.stringify
      const result = await addDocumentsBulk(JSON.stringify(newDocs));
      console.log("Bulk add result:", result);

      // Save to AsyncStorage
      const existingDocs = await getDocumentsFromStorage();
      existingDocs.push(...newDocs);
      await saveDocumentsToStorage(existingDocs);

      // Update count
      await fetchCount();

      Alert.alert("Success", `${newDocs.length} documents added!`);
    } catch (error) {
      console.error("Add error:", error);
      Alert.alert("Error", String(error));
    }
  };

  const onSearch = async (text: string) => {
    setSearchQuery(text);

    if (!initialized) return;
    if (!text.trim()) {
      setResults([]);
      return;
    }

    try {
      // 5. FIXED: Parse the JSON result from Rust
      // searchDocuments returns a JSON string, not an object
      const resultString = await searchDocuments(text);
      const searchResults: SearchResult[] = JSON.parse(resultString);

      console.log("Search results:", searchResults);
      setResults(searchResults);
    } catch (error) {
      console.error("Search error:", error);
      setResults([]);
    }
  };

  const onClearAll = async () => {
    Alert.alert(
      "Clear All Data",
      "This will delete all documents. Are you sure?",
      [
        { text: "Cancel", style: "cancel" },
        {
          text: "Clear",
          style: "destructive",
          onPress: async () => {
            try {
              await AsyncStorage.removeItem(STORAGE_KEY);
              // Call the native clear function if you have one exposed,
              // otherwise re-initialize wipes the in-memory index
              await initializeIndex();

              setCount(0);
              setResults([]);
              setSearchQuery("");
              Alert.alert("Success", "All documents cleared");
            } catch (error) {
              console.error("Clear error:", error);
              Alert.alert("Error", String(error));
            }
          },
        },
      ]
    );
  };

  if (!initialized) {
    return (
      <View style={styles.container}>
        <Text>Initializing search index...</Text>
        <Text style={styles.subtext}>{initMessage}</Text>
      </View>
    );
  }

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.contentContainer}
    >
      <Text style={styles.title}>Rust Search Demo</Text>
      <Text style={styles.subtext}>Documents in index: {count}</Text>
      <Text style={styles.subtext}>{initMessage}</Text>

      <View style={styles.buttonRow}>
        <Button title="Add 1 Doc" onPress={onAddDocument} />
        <Button title="Add 3 Docs" onPress={onAddMultipleDocuments} />
        <Button title="Clear All" onPress={onClearAll} color="red" />
      </View>

      <TextInput
        style={styles.input}
        placeholder="Search documents... (try: rust, react, search)"
        value={searchQuery}
        onChangeText={onSearch}
      />

      {results.length > 0 && (
        <Text style={styles.resultsHeader}>
          Found {results.length} result{results.length !== 1 ? "s" : ""}:
        </Text>
      )}

      <FlatList
        data={results}
        keyExtractor={(item, index) => `${item.title}-${index}`}
        scrollEnabled={false}
        renderItem={({ item }) => (
          <View style={styles.resultItem}>
            <Text style={styles.resultTitle}>{item.title}</Text>
            <Text style={styles.resultBody}>{item.body}</Text>
            <Text style={styles.resultScore}>
              Score: {item.score.toFixed(2)}
            </Text>
          </View>
        )}
        ListEmptyComponent={
          searchQuery ? (
            <Text style={styles.emptyText}>No results found</Text>
          ) : null
        }
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#fff",
  },
  contentContainer: {
    padding: 20,
    paddingTop: 60,
    gap: 15,
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    marginBottom: 10,
  },
  subtext: {
    fontSize: 14,
    color: "#666",
  },
  buttonRow: {
    flexDirection: "row",
    gap: 10,
    marginVertical: 10,
  },
  input: {
    borderWidth: 1,
    borderColor: "#ddd",
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
  },
  resultsHeader: {
    fontSize: 16,
    fontWeight: "600",
    marginTop: 10,
  },
  resultItem: {
    padding: 15,
    borderWidth: 1,
    borderColor: "#ddd",
    borderRadius: 8,
    marginTop: 10,
    backgroundColor: "#f9f9f9",
  },
  resultTitle: {
    fontSize: 18,
    fontWeight: "bold",
    marginBottom: 5,
  },
  resultBody: {
    fontSize: 14,
    color: "#333",
    marginBottom: 5,
  },
  resultScore: {
    fontSize: 12,
    color: "#999",
    fontStyle: "italic",
  },
  emptyText: {
    fontSize: 14,
    color: "#999",
    textAlign: "center",
    marginTop: 20,
  },
});
