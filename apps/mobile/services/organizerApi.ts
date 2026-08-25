import AsyncStorage from '@react-native-async-storage/async-storage';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8080/api/v1';

export interface GateMetrics {
  gate_id: string;
  gate_name: string;
  current_wait_time_minutes: number;
  throughput_per_minute: number;
  staff_count: number;
  congestion_level: 'low' | 'medium' | 'high' | 'critical';
}

export interface ScanVelocityEvent {
  event_id: string;
  gate_id: string;
  scans_per_minute: number;
  total_scans: number;
  timestamp: string;
}

export interface GateHeatmapEvent {
  event_id: string;
  gates: GateMetrics[];
  timestamp: string;
}

export interface StaffAuthEvent {
  event_id: string;
  staff_wallet: string;
  gate_id: string;
  authorized: boolean;
  timestamp: string;
}

export interface OrganizerEvent {
  type: 'ScanVelocity' | 'GateHeatmap' | 'StaffAuth';
  data?: any;
}

export interface DelegationQRCode {
  code: string;
  event_id: string;
  gate_id: string;
  expires_at: string;
  permissions: string[];
}

export interface DoorTicketRequest {
  event_id: string;
  ticket_tier_id: string;
  quantity: number;
  payment_method: 'usdc' | 'qr';
  attendee_email?: string;
  attendee_name?: string;
}

export interface DoorTicketResponse {
  success: boolean;
  tickets: Array<{
    id: string;
    qr_code: string;
    ticket_tier_id: string;
    attendee_name: string;
  }>;
  transaction_id?: string;
  error?: string;
}

export interface BroadcastAlert {
  event_id: string;
  message: string;
  alert_type: 'info' | 'urgent' | 'emergency';
}

class OrganizerApiService {
  private async getAuthHeaders(): Promise<HeadersInit> {
    const token = await AsyncStorage.getItem('auth_token');
    return {
      'Content-Type': 'application/json',
      ...(token && { Authorization: `Bearer ${token}` }),
    };
  }

  // Generate delegation QR code for staff scanner authorization
  async generateDelegationQR(
    eventId: string,
    gateId: string,
    staffWallet: string,
    expiresInMinutes: number = 480
  ): Promise<DelegationQRCode> {
    const headers = await this.getAuthHeaders();
    const response = await fetch(`${API_BASE}/organizer/delegation`, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        event_id: eventId,
        gate_id: gateId,
        staff_wallet: staffWallet,
        expires_in_minutes: expiresInMinutes,
      }),
    });

    if (!response.ok) {
      throw new Error('Failed to generate delegation QR code');
    }

    return response.json();
  }

  // Verify delegation QR code (for staff scanning)
  async verifyDelegationQR(code: string): Promise<{
    valid: boolean;
    event_id: string;
    gate_id: string;
    permissions: string[];
  }> {
    const headers = await this.getAuthHeaders();
    const response = await fetch(`${API_BASE}/organizer/delegation/verify`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ code }),
    });

    if (!response.ok) {
      throw new Error('Failed to verify delegation QR code');
    }

    return response.json();
  }

  // Create door ticket with instant payment
  async createDoorTicket(request: DoorTicketRequest): Promise<DoorTicketResponse> {
    const headers = await this.getAuthHeaders();
    const response = await fetch(`${API_BASE}/organizer/door-ticket`, {
      method: 'POST',
      headers,
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error('Failed to create door ticket');
    }

    return response.json();
  }

  // Send broadcast alert to checked-in attendees
  async sendBroadcastAlert(alert: BroadcastAlert): Promise<{ success: boolean; message?: string; error?: string }> {
    const headers = await this.getAuthHeaders();
    const response = await fetch(`${API_BASE}/organizer/broadcast`, {
      method: 'POST',
      headers,
      body: JSON.stringify(alert),
    });

    if (!response.ok) {
      throw new Error('Failed to send broadcast alert');
    }

    return response.json();
  }

  // Get event gates configuration
  async getEventGates(eventId: string): Promise<GateMetrics[]> {
    const headers = await this.getAuthHeaders();
    const response = await fetch(`${API_BASE}/events/${eventId}/gates`, {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      throw new Error('Failed to fetch event gates');
    }

    return response.json();
  }

  // Get active staff authorizations for an event
  async getStaffAuthorizations(eventId: string): Promise<StaffAuthEvent[]> {
    const headers = await this.getAuthHeaders();
    const response = await fetch(`${API_BASE}/organizer/${eventId}/staff`, {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      throw new Error('Failed to fetch staff authorizations');
    }

    return response.json();
  }

  // Revoke staff authorization
  async revokeStaffAuthorization(eventId: string, staffWallet: string): Promise<{ success: boolean }> {
    const headers = await this.getAuthHeaders();
    const response = await fetch(`${API_BASE}/organizer/${eventId}/staff/revoke`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ staff_wallet: staffWallet }),
    });

    if (!response.ok) {
      throw new Error('Failed to revoke staff authorization');
    }

    return response.json();
  }
}

export const organizerApi = new OrganizerApiService();
