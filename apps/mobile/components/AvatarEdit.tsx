import React, { useState } from 'react';
import {
  View,
  Image,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Text,
} from 'react-native';
import * as ImagePicker from 'expo-image-picker';
import Colors from '@/constants/Colors';

interface AvatarEditProps {
  initialImage?: string | null;
  onImageSelected: (uri: string) => void;
  userName?: string;
}

export const AvatarEdit: React.FC<AvatarEditProps> = ({
  initialImage,
  onImageSelected,
  userName,
}) => {
  const [image, setImage] = useState<string | null | undefined>(initialImage);

  const pickImage = async () => {
    // Request permission
    const { status } = await ImagePicker.requestMediaLibraryPermissionsAsync();
    
    if (status !== 'granted') {
      Alert.alert(
        'Permission Denied',
        'Sorry, we need camera roll permissions to make this work!',
        [
          { text: 'Cancel', style: 'cancel' },
          { text: 'OK' }
        ]
      );
      return;
    }

    const result = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      aspect: [1, 1],
      quality: 1,
    });

    if (!result.canceled) {
      const uri = result.assets[0].uri;
      setImage(uri);
      onImageSelected(uri);
    }
  };

  return (
    <TouchableOpacity style={styles.container} onPress={pickImage} testID="avatar-edit-button">
      {image ? (
        <Image source={{ uri: image }} style={styles.avatar} />
      ) : (
        <View style={styles.avatarPlaceholder}>
          <Text style={styles.avatarText}>
            {userName ? userName.charAt(0).toUpperCase() : 'A'}
          </Text>
        </View>
      )}
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    marginBottom: 16,
  },
  avatar: {
    width: 90,
    height: 90,
    borderRadius: 45,
  },
  avatarPlaceholder: {
    width: 90,
    height: 90,
    borderRadius: 45,
    backgroundColor: Colors.primaryYellow,
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    fontSize: 32,
    fontWeight: 'bold',
    color: Colors.darkBackground,
  },
});

export default AvatarEdit;
