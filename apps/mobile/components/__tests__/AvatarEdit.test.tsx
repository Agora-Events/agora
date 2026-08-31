import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import * as ImagePicker from 'expo-image-picker';
import AvatarEdit from '../AvatarEdit';

jest.mock('expo-image-picker', () => ({
  requestMediaLibraryPermissionsAsync: jest.fn(),
  launchImageLibraryAsync: jest.fn(),
  MediaTypeOptions: { Images: 'Images' },
}));

describe('AvatarEdit', () => {
  it('renders the placeholder correctly', () => {
    const { getByText } = render(<AvatarEdit onImageSelected={jest.fn()} userName="John" />);
    expect(getByText('J')).toBeTruthy();
  });

  it('calls ImagePicker.requestMediaLibraryPermissionsAsync when pressed', async () => {
    const mockRequest = ImagePicker.requestMediaLibraryPermissionsAsync as jest.Mock;
    mockRequest.mockResolvedValue({ status: 'granted' });

    const mockLaunch = ImagePicker.launchImageLibraryAsync as jest.Mock;
    mockLaunch.mockResolvedValue({ canceled: true });

    const { getByTestId } = render(<AvatarEdit onImageSelected={jest.fn()} />);

    fireEvent.press(getByTestId('avatar-edit-button'));

    await waitFor(() => {
      expect(mockRequest).toHaveBeenCalled();
    });
  });

  it('calls onImageSelected when an image is picked', async () => {
    const mockRequest = ImagePicker.requestMediaLibraryPermissionsAsync as jest.Mock;
    mockRequest.mockResolvedValue({ status: 'granted' });

    const mockLaunch = ImagePicker.launchImageLibraryAsync as jest.Mock;
    mockLaunch.mockResolvedValue({ 
      canceled: false, 
      assets: [{ uri: 'file://test.jpg' }] 
    });

    const onImageSelected = jest.fn();
    const { getByTestId } = render(<AvatarEdit onImageSelected={onImageSelected} />);

    fireEvent.press(getByTestId('avatar-edit-button'));

    await waitFor(() => {
      expect(onImageSelected).toHaveBeenCalledWith('file://test.jpg');
    });
  });
});
