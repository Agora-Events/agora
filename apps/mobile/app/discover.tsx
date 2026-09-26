import React, { useMemo, useState } from "react";
import {
  ActivityIndicator,
  FlatList,
  Image,
  RefreshControl,
  StyleSheet,
  Text,
  TextInput,
  TouchableOpacity,
  View,
} from "react-native";
import MapView, { Marker, Callout } from "react-native-maps";
import CategorySelector from "@/components/CategorySelector";
import { useDiscoverEvents } from "@/hooks/useDiscoverEvents";
import {
  mappableEvents,
  regionForEvents,
  type DiscoverEvent,
} from "@/lib/discoverFilters";

/**
 * Event discovery screen (issue #1004).
 *
 * List and map render from the same filtered array, so a category tap refines
 * both in the same frame — the acceptance criterion — rather than relying on
 * two code paths applying the same filter consistently.
 */

type ViewMode = "list" | "map";

function EventRow({ event }: { event: DiscoverEvent }) {
  return (
    <View style={styles.row}>
      {event.imageUrl ? (
        <Image source={{ uri: event.imageUrl }} style={styles.thumb} />
      ) : (
        <View style={[styles.thumb, styles.thumbPlaceholder]} />
      )}
      <View style={styles.rowBody}>
        <Text style={styles.rowTitle} numberOfLines={1}>
          {event.title}
        </Text>
        <Text style={styles.rowMeta} numberOfLines={1}>
          {event.category}
          {event.venue ? ` · ${event.venue}` : ""}
        </Text>
        <Text style={styles.rowMeta}>
          {new Date(event.startsAt).toLocaleDateString()}
        </Text>
      </View>
    </View>
  );
}

export default function DiscoverScreen() {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<string | null>(null);
  const [mode, setMode] = useState<ViewMode>("list");

  const { events, isLoading, isRefreshing, isOffline, refetch } =
    useDiscoverEvents({ query, category });

  const pins = useMemo(() => mappableEvents(events), [events]);
  // Recomputed from the filtered set, so the map reframes to whatever the
  // active filter left visible instead of staying on the unfiltered bounds.
  const region = useMemo(() => regionForEvents(events), [events]);

  return (
    <View style={styles.screen}>
      <View style={styles.header}>
        <TextInput
          value={query}
          onChangeText={setQuery}
          placeholder="Search events"
          placeholderTextColor="#71717A"
          style={styles.search}
          accessibilityLabel="Search events"
          testID="discover-search"
        />
        <TouchableOpacity
          onPress={() => setMode((m) => (m === "list" ? "map" : "list"))}
          style={styles.toggle}
          accessibilityRole="button"
          accessibilityLabel={mode === "list" ? "Switch to map view" : "Switch to list view"}
          testID="discover-view-toggle"
        >
          <Text style={styles.toggleText}>{mode === "list" ? "Map" : "List"}</Text>
        </TouchableOpacity>
      </View>

      <CategorySelector selected={category} onSelect={setCategory} />

      {isOffline && (
        <View style={styles.offlineBanner} testID="discover-offline-banner">
          <Text style={styles.offlineText}>
            Showing saved events — you appear to be offline
          </Text>
        </View>
      )}

      {isLoading ? (
        <ActivityIndicator style={styles.loader} color="#FACC15" />
      ) : mode === "list" ? (
        <FlatList
          data={events}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => <EventRow event={item} />}
          refreshControl={
            <RefreshControl refreshing={isRefreshing} onRefresh={refetch} tintColor="#FACC15" />
          }
          ListEmptyComponent={
            <Text style={styles.empty}>No events match your filters.</Text>
          }
          contentContainerStyle={events.length === 0 ? styles.emptyContainer : undefined}
          testID="discover-list"
        />
      ) : (
        <MapView style={styles.map} region={region} testID="discover-map">
          {pins.map((event) => (
            <Marker
              key={event.id}
              coordinate={{
                latitude: event.latitude as number,
                longitude: event.longitude as number,
              }}
              title={event.title}
            >
              <Callout>
                <View style={styles.callout}>
                  {event.imageUrl ? (
                    <Image source={{ uri: event.imageUrl }} style={styles.calloutThumb} />
                  ) : null}
                  <Text style={styles.calloutTitle} numberOfLines={1}>
                    {event.title}
                  </Text>
                  <Text style={styles.calloutMeta} numberOfLines={1}>
                    {event.category}
                  </Text>
                </View>
              </Callout>
            </Marker>
          ))}
        </MapView>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  screen: { flex: 1, backgroundColor: "#09090B" },
  header: { flexDirection: "row", alignItems: "center", gap: 8, padding: 16 },
  search: {
    flex: 1,
    backgroundColor: "#18181B",
    borderRadius: 10,
    paddingHorizontal: 12,
    paddingVertical: 10,
    color: "#FFFFFF",
  },
  toggle: {
    backgroundColor: "#FACC15",
    borderRadius: 10,
    paddingHorizontal: 14,
    paddingVertical: 10,
  },
  toggleText: { color: "#000000", fontWeight: "700" },
  offlineBanner: {
    backgroundColor: "#422006",
    paddingHorizontal: 16,
    paddingVertical: 6,
  },
  offlineText: { color: "#FACC15", fontSize: 12 },
  loader: { marginTop: 32 },
  map: { flex: 1 },
  row: { flexDirection: "row", gap: 12, paddingHorizontal: 16, paddingVertical: 10 },
  thumb: { width: 64, height: 64, borderRadius: 8 },
  thumbPlaceholder: { backgroundColor: "#27272A" },
  rowBody: { flex: 1, justifyContent: "center" },
  rowTitle: { color: "#FFFFFF", fontSize: 15, fontWeight: "600" },
  rowMeta: { color: "#A1A1AA", fontSize: 12, marginTop: 2 },
  empty: { color: "#A1A1AA", textAlign: "center" },
  emptyContainer: { flexGrow: 1, justifyContent: "center" },
  callout: { width: 160 },
  calloutThumb: { width: "100%", height: 72, borderRadius: 6, marginBottom: 6 },
  calloutTitle: { fontWeight: "700", fontSize: 13 },
  calloutMeta: { fontSize: 11, color: "#52525B" },
});
