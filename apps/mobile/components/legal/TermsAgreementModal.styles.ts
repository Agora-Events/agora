import { StyleSheet } from 'react-native';
import Colors from '@/constants/Colors';

export default StyleSheet.create({
  overlay: {
    flex: 1,
    justifyContent: 'center',
    padding: 20,
    backgroundColor: 'rgba(0, 0, 0, 0.75)',
  },
  card: {
    maxHeight: '82%',
    padding: 22,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    borderRadius: 16,
    backgroundColor: '#1E1E20',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  title: { color: Colors.primaryText, fontSize: 20, fontWeight: '700' },
  close: { color: Colors.primaryYellow, fontSize: 14, fontWeight: '600' },
  scrollView: { maxHeight: 360 },
  scrollContent: { paddingRight: 8, paddingBottom: 4 },
  terms: { color: Colors.primaryText, fontSize: 14, lineHeight: 22 },
  hint: {
    marginVertical: 14,
    color: Colors.secondaryText,
    fontSize: 12,
    textAlign: 'center',
  },
});
