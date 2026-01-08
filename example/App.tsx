import React, { useState } from "react";
import {
  View,
  Text,
  TextInput,
  Button,
  FlatList,
  StyleSheet,
} from "react-native";
import Search from "expo-rusty-search";

export default function App() {
  const [results, setResults] = useState([]);
  const [count, setCount] = useState("0");

  const onAdd = async () => {
    // 1. addDocument
    await Search.addDocument("Test Title", "This is a string sent to Rust");
  };

  const onGetCount = async () => {
    // 2. getDocumentCount
    const c = await Search.getDocumentCount();
    setCount(c);
  };

  const onSearch = async (text: string) => {
    // 3. search
    const rawJson = await Search.search(text);
    setResults(JSON.parse(rawJson));
  };

  return (
    <View style={styles.container}>
      <Text style={styles.text}>Docs in Rust: {count}</Text>

      <Button title="Add Document" onPress={onAdd} />
      <Button title="Get Count" onPress={onGetCount} />

      <TextInput
        style={styles.input}
        placeholder="Search..."
        onChangeText={onSearch}
      />

      <FlatList
        data={results}
        renderItem={({ item }) => (
          <View style={styles.item}>
            <Text>{item}</Text>
          </View>
        )}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 60, gap: 20 },
  text: { fontSize: 20, fontWeight: "bold" },
  input: { borderBottomWidth: 1, padding: 10 },
  item: { padding: 10, borderBottomWidth: 1, borderColor: "#ccc" },
});
