import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  ScrollView,
  Pressable,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import Input from '@/components/ui/Input';
import { useAuth } from '@/hooks/useAuth';
import ScanVelocityChart from '@/components/organizer/ScanVelocityChart';
import {
  organizerApi,
  GateMetrics,
  ScanVelocityEvent,
  GateHeatmapEvent,
  StaffAuthEvent,
  DelegationQRCode,
  BroadcastAlert,
} from '@/services/organizerApi';

export default function OrganizerDashboard() {
  const { user } = useAuth();
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);
  const [scanVelocityData, setScanVelocityData] = useState<ScanVelocityEvent[]>([]);
  const [gateMetrics, setGateMetrics] = useState<GateMetrics[]>([]);
  const [staffAuthorizations, setStaffAuthorizations] = useState<StaffAuthEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [wsConnected, setWsConnected] = useState(false);

  // Delegation QR code state
  const [showDelegationModal, setShowDelegationModal] = useState(false);
  const [delegationGateId, setDelegationGateId] = useState('');
  const [delegationStaffWallet, setDelegationStaffWallet] = useState('');
  const [generatedQR, setGeneratedQR] = useState<DelegationQRCode | null>(null);

  // Door sales state
  const [showDoorSalesModal, setShowDoorSalesModal] = useState(false);
  const [doorSalesTierId, setDoorSalesTierId] = useState('');
  const [doorSalesQuantity, setDoorSalesQuantity] = useState('1');
  const [doorSalesEmail, setDoorSalesEmail] = useState('');
  const [doorSalesName, setDoorSalesName] = useState('');

  // Broadcast alert state
  const [showBroadcastModal, setShowBroadcastModal] = useState(false);
  const [broadcastMessage, setBroadcastMessage] = useState('');
  const [broadcastType, setBroadcastType] = useState<'info' | 'urgent' | 'emergency'>('info');

  const fetchEventData = useCallback(async () => {
    if (!selectedEventId) return;
    
    try {
      setLoading(true);
      const [gates, staff] = await Promise.all([
        organizerApi.getEventGates(selectedEventId),
        organizerApi.getStaffAuthorizations(selectedEventId),
      ]);
      setGateMetrics(gates);
      setStaffAuthorizations(staff);
    } catch (error) {
      console.error('Failed to fetch event data:', error);
    } finally {
      setLoading(false);
    }
  }, [selectedEventId]);

  useEffect(() => {
    if (selectedEventId) {
      fetchEventData();
    }
  }, [selectedEventId, fetchEventData]);

  // WebSocket connection for live telemetry
  useEffect(() => {
    if (!selectedEventId || !user?.walletAddress) return;

    const wsUrl = `${process.env.EXPO_PUBLIC_WS_URL || 'ws://localhost:8080'}/api/v1/ws/organizer?token=${user.authToken}`;
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      setWsConnected(true);
      console.log('Organizer WebSocket connected');
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        
        if (data.type === 'ScanVelocity') {
          setScanVelocityData((prev) => {
            const newData = [...prev, data.data];
            return newData.slice(-60); // Keep last 60 data points
          });
        } else if (data.type === 'GateHeatmap') {
          setGateMetrics(data.data.gates);
        } else if (data.type === 'StaffAuth') {
          setStaffAuthorizations((prev) => {
            const existing = prev.findIndex(
              (s) => s.staff_wallet === data.data.staff_wallet
            );
            if (existing >= 0) {
              const updated = [...prev];
              updated[existing] = data.data;
              return updated;
            }
            return [...prev, data.data];
          });
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setWsConnected(false);
    };

    ws.onclose = () => {
      setWsConnected(false);
      console.log('Organizer WebSocket disconnected');
    };

    return () => {
      ws.close();
    };
  }, [selectedEventId, user?.walletAddress, user?.authToken]);

  const onRefresh = async () => {
    setRefreshing(true);
    await fetchEventData();
    setRefreshing(false);
  };

  const handleGenerateDelegation = async () => {
    if (!selectedEventId || !delegationGateId || !delegationStaffWallet) return;

    try {
      const qr = await organizerApi.generateDelegationQR(
        selectedEventId,
        delegationGateId,
        delegationStaffWallet
      );
      setGeneratedQR(qr);
      setShowDelegationModal(false);
    } catch (error) {
      console.error('Failed to generate delegation QR:', error);
    }
  };

  const handleCreateDoorTicket = async () => {
    if (!selectedEventId) return;

    try {
      await organizerApi.createDoorTicket({
        event_id: selectedEventId,
        ticket_tier_id: doorSalesTierId,
        quantity: parseInt(doorSalesQuantity) || 1,
        payment_method: 'usdc',
        attendee_email: doorSalesEmail,
        attendee_name: doorSalesName,
      });
      setShowDoorSalesModal(false);
      // Reset form
      setDoorSalesTierId('');
      setDoorSalesQuantity('1');
      setDoorSalesEmail('');
      setDoorSalesName('');
    } catch (error) {
      console.error('Failed to create door ticket:', error);
    }
  };

  const handleSendBroadcast = async () => {
    if (!selectedEventId || !broadcastMessage) return;

    try {
      await organizerApi.sendBroadcastAlert({
        event_id: selectedEventId,
        message: broadcastMessage,
        alert_type: broadcastType,
      });
      setShowBroadcastModal(false);
      setBroadcastMessage('');
    } catch (error) {
      console.error('Failed to send broadcast:', error);
    }
  };

  const getCongestionColor = (level: string) => {
    switch (level) {
      case 'low':
        return '#34C759';
      case 'medium':
        return '#FF9500';
      case 'high':
        return '#FF3B30';
      case 'critical':
        return '#FF0000';
      default:
        return '#8E8E93';
    }
  };

  return (
    <SafeAreaView style={styles.container}>
      <ScrollView
        contentContainerStyle={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        {/* Header */}
        <View style={styles.header}>
          <Text style={styles.headerTitle}>Organizer Dashboard</Text>
          <View style={styles.connectionStatus}>
            <View
              style={[
                styles.statusDot,
                { backgroundColor: wsConnected ? '#34C759' : '#FF3B30' },
              ]}
            />
            <Text style={styles.statusText}>
              {wsConnected ? 'Live' : 'Disconnected'}
            </Text>
          </View>
        </View>

        {/* Event Selector */}
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Select Event</Text>
          <Input
            placeholder="Enter Event ID"
            value={selectedEventId || ''}
            onChangeText={setSelectedEventId}
          />
        </View>

        {selectedEventId && (
          <>
            {/* Live Scan Velocity Chart */}
            <View style={styles.card}>
              <View style={styles.cardHeader}>
                <Text style={styles.cardTitle}>Scan Velocity (scans/min)</Text>
                <Text style={styles.liveIndicator}>LIVE</Text>
              </View>
              {scanVelocityData.length > 0 ? (
                <ScanVelocityChart
                  data={scanVelocityData.map((d) => ({
                    timestamp: d.timestamp,
                    value: d.scans_per_minute,
                  }))}
                />
              ) : (
                <View style={styles.emptyState}>
                  <ActivityIndicator color={Colors.primaryYellow} />
                  <Text style={styles.emptyText}>Waiting for live data...</Text>
                </View>
              )}
            </View>

            {/* Gate Heatmap */}
            <View style={styles.card}>
              <Text style={styles.cardTitle}>Gate Congestion Heatmap</Text>
              {loading ? (
                <ActivityIndicator color={Colors.primaryYellow} />
              ) : gateMetrics.length === 0 ? (
                <Text style={styles.emptyText}>No gate data available</Text>
              ) : (
                <View style={styles.gateGrid}>
                  {gateMetrics.map((gate) => (
                    <View key={gate.gate_id} style={styles.gateCard}>
                      <Text style={styles.gateName}>{gate.gate_name}</Text>
                      <View style={styles.gateMetrics}>
                        <View style={styles.metricRow}>
                          <Text style={styles.metricLabel}>Wait Time:</Text>
                          <Text style={styles.metricValue}>
                            {gate.current_wait_time_minutes.toFixed(1)} min
                          </Text>
                        </View>
                        <View style={styles.metricRow}>
                          <Text style={styles.metricLabel}>Throughput:</Text>
                          <Text style={styles.metricValue}>
                            {gate.throughput_per_minute.toFixed(1)}/min
                          </Text>
                        </View>
                        <View style={styles.metricRow}>
                          <Text style={styles.metricLabel}>Staff:</Text>
                          <Text style={styles.metricValue}>{gate.staff_count}</Text>
                        </View>
                      </View>
                      <View
                        style={[
                          styles.congestionBadge,
                          { backgroundColor: `${getCongestionColor(gate.congestion_level)}33` },
                        ]}
                      >
                        <Text
                          style={[
                            styles.congestionText,
                            { color: getCongestionColor(gate.congestion_level) },
                          ]}
                        >
                          {gate.congestion_level.toUpperCase()}
                        </Text>
                      </View>
                    </View>
                  ))}
                </View>
              )}
            </View>

            {/* Staff Authorizations */}
            <View style={styles.card}>
              <View style={styles.cardHeader}>
                <Text style={styles.cardTitle}>Staff Scanner Authorizations</Text>
                <Pressable onPress={() => setShowDelegationModal(true)}>
                  <Text style={styles.addButton}>+ Add Staff</Text>
                </Pressable>
              </View>
              {loading ? (
                <ActivityIndicator color={Colors.primaryYellow} />
              ) : staffAuthorizations.length === 0 ? (
                <Text style={styles.emptyText}>No staff authorized</Text>
              ) : (
                staffAuthorizations.map((staff) => (
                  <View key={staff.staff_wallet} style={styles.staffRow}>
                    <View style={styles.staffMain}>
                      <Text style={styles.staffWallet}>
                        {staff.staff_wallet.slice(0, 8)}...{staff.staff_wallet.slice(-4)}
                      </Text>
                      <Text style={styles.staffGate}>Gate: {staff.gate_id}</Text>
                    </View>
                    <View
                      style={[
                        styles.authBadge,
                        { backgroundColor: staff.authorized ? '#34C75922' : '#FF3B3022' },
                      ]}
                    >
                      <Text
                        style={[
                          styles.authText,
                          { color: staff.authorized ? '#34C759' : '#FF3B30' },
                        ]}
                      >
                        {staff.authorized ? 'Authorized' : 'Revoked'}
                      </Text>
                    </View>
                  </View>
                ))
              )}
            </View>

            {/* Quick Actions */}
            <View style={styles.card}>
              <Text style={styles.cardTitle}>Quick Actions</Text>
              <View style={styles.actionButtons}>
                <Button
                  title="Door Sales"
                  onPress={() => setShowDoorSalesModal(true)}
                  style={styles.actionButton}
                />
                <Button
                  title="Send Broadcast"
                  variant="secondary"
                  onPress={() => setShowBroadcastModal(true)}
                  style={styles.actionButton}
                />
              </View>
            </View>
          </>
        )}

        {/* Delegation Modal */}
        {showDelegationModal && (
          <View style={styles.modal}>
            <View style={styles.modalContent}>
              <Text style={styles.modalTitle}>Generate Staff Delegation</Text>
              <Input
                label="Gate ID"
                placeholder="Enter gate ID"
                value={delegationGateId}
                onChangeText={setDelegationGateId}
              />
              <Input
                label="Staff Wallet Address"
                placeholder="Enter wallet address"
                value={delegationStaffWallet}
                onChangeText={setDelegationStaffWallet}
              />
              <View style={styles.modalButtons}>
                <Button
                  title="Cancel"
                  variant="outline"
                  onPress={() => setShowDelegationModal(false)}
                  style={styles.modalButton}
                />
                <Button
                  title="Generate QR"
                  onPress={handleGenerateDelegation}
                  style={styles.modalButton}
                />
              </View>
            </View>
          </View>
        )}

        {/* Door Sales Modal */}
        {showDoorSalesModal && (
          <View style={styles.modal}>
            <View style={styles.modalContent}>
              <Text style={styles.modalTitle}>Door Ticket Sale</Text>
              <Input
                label="Ticket Tier ID"
                placeholder="Enter ticket tier ID"
                value={doorSalesTierId}
                onChangeText={setDoorSalesTierId}
              />
              <Input
                label="Quantity"
                placeholder="1"
                keyboardType="numeric"
                value={doorSalesQuantity}
                onChangeText={setDoorSalesQuantity}
              />
              <Input
                label="Attendee Email"
                placeholder="email@example.com"
                value={doorSalesEmail}
                onChangeText={setDoorSalesEmail}
              />
              <Input
                label="Attendee Name"
                placeholder="Full name"
                value={doorSalesName}
                onChangeText={setDoorSalesName}
              />
              <View style={styles.modalButtons}>
                <Button
                  title="Cancel"
                  variant="outline"
                  onPress={() => setShowDoorSalesModal(false)}
                  style={styles.modalButton}
                />
                <Button
                  title="Create Ticket"
                  onPress={handleCreateDoorTicket}
                  style={styles.modalButton}
                />
              </View>
            </View>
          </View>
        )}

        {/* Broadcast Modal */}
        {showBroadcastModal && (
          <View style={styles.modal}>
            <View style={styles.modalContent}>
              <Text style={styles.modalTitle}>Send Broadcast Alert</Text>
              <Input
                label="Message"
                placeholder="Enter your message"
                multiline
                numberOfLines={4}
                value={broadcastMessage}
                onChangeText={setBroadcastMessage}
              />
              <Text style={styles.label}>Alert Type</Text>
              <View style={styles.alertTypes}>
                {(['info', 'urgent', 'emergency'] as const).map((type) => (
                  <Pressable
                    key={type}
                    style={[
                      styles.alertTypeButton,
                      broadcastType === type && styles.alertTypeButtonActive,
                    ]}
                    onPress={() => setBroadcastType(type)}
                  >
                    <Text
                      style={[
                        styles.alertTypeText,
                        broadcastType === type && styles.alertTypeTextActive,
                      ]}
                    >
                      {type.toUpperCase()}
                    </Text>
                  </Pressable>
                ))}
              </View>
              <View style={styles.modalButtons}>
                <Button
                  title="Cancel"
                  variant="outline"
                  onPress={() => setShowBroadcastModal(false)}
                  style={styles.modalButton}
                />
                <Button
                  title="Send Broadcast"
                  onPress={handleSendBroadcast}
                  style={styles.modalButton}
                />
              </View>
            </View>
          </View>
        )}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    padding: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 20,
  },
  headerTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: Colors.primaryText,
  },
  connectionStatus: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 6,
  },
  statusText: {
    fontSize: 12,
    color: Colors.secondaryText,
  },
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: Colors.primaryText,
  },
  liveIndicator: {
    fontSize: 12,
    fontWeight: 'bold',
    color: '#34C759',
    backgroundColor: '#34C75922',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 8,
  },
  emptyState: {
    alignItems: 'center',
    paddingVertical: 24,
  },
  emptyText: {
    color: Colors.secondaryText,
    marginTop: 8,
  },
  gateGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginHorizontal: -4,
  },
  gateCard: {
    width: '48%',
    marginHorizontal: '1%',
    backgroundColor: '#141416',
    borderRadius: 8,
    padding: 12,
    marginBottom: 8,
  },
  gateName: {
    fontSize: 14,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 8,
  },
  gateMetrics: {
    marginBottom: 8,
  },
  metricRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 4,
  },
  metricLabel: {
    fontSize: 12,
    color: Colors.secondaryText,
  },
  metricValue: {
    fontSize: 12,
    fontWeight: 'bold',
    color: Colors.primaryText,
  },
  congestionBadge: {
    alignSelf: 'flex-start',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
  },
  congestionText: {
    fontSize: 10,
    fontWeight: 'bold',
  },
  addButton: {
    fontSize: 14,
    color: Colors.primaryYellow,
    fontWeight: 'bold',
  },
  staffRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: '#2C2C2E',
  },
  staffMain: {
    flex: 1,
  },
  staffWallet: {
    fontSize: 14,
    fontWeight: 'bold',
    color: Colors.primaryText,
    fontFamily: 'SpaceMono',
  },
  staffGate: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginTop: 2,
  },
  authBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 8,
  },
  authText: {
    fontSize: 12,
    fontWeight: 'bold',
  },
  actionButtons: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  actionButton: {
    flex: 1,
    marginHorizontal: 4,
  },
  modal: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalContent: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 20,
    width: '90%',
    maxWidth: 400,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    color: Colors.primaryText,
    marginBottom: 8,
    marginTop: 8,
  },
  alertTypes: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  alertTypeButton: {
    flex: 1,
    padding: 10,
    borderRadius: 8,
    backgroundColor: '#141416',
    borderWidth: 1,
    borderColor: '#2C2C2E',
    marginRight: 8,
  },
  alertTypeButtonActive: {
    backgroundColor: Colors.primaryYellow,
    borderColor: Colors.primaryYellow,
  },
  alertTypeText: {
    fontSize: 12,
    fontWeight: 'bold',
    color: Colors.primaryText,
    textAlign: 'center',
  },
  alertTypeTextActive: {
    color: '#000',
  },
  modalButtons: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  modalButton: {
    flex: 1,
    marginHorizontal: 4,
  },
});
