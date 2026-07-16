import { create } from 'zustand';

export interface UserProfile {
  name: string;
  email: string;
  walletAddress: string;
}

export interface AuthState {
  token: string | null;
  user: UserProfile | null;
  isAuthenticated: boolean;
  login: (email: string, name?: string, walletAddress?: string) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  token: null,
  user: null,
  isAuthenticated: false,
  login: (email: string, name: string = 'Agora User', walletAddress: string = 'GDAGORA...') => {
    set({
      token: 'mock-jwt-token-agora',
      user: { name, email, walletAddress },
      isAuthenticated: true,
    });
  },
  logout: () => {
    set({
      token: null,
      user: null,
      isAuthenticated: false,
    });
  },
}));

export const useAuth = () => {
  const store = useAuthStore();
  return store;
};
