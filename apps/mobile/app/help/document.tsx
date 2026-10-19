import React, { useMemo, useState } from 'react';
import {
  ScrollView,
  StyleSheet,
  Text,
  View,
  TouchableOpacity,
  Linking,
  Platform,
} from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useThemeContext } from '@/context/ThemeContext';
import { Colors } from '@/constants/Colors';
import { Ionicons } from '@expo/vector-icons';

interface PolicyDocument {
  id: string;
  title: string;
  lastUpdated: string;
  content: string;
}

const DEFAULT_DOCUMENTS: Record<string, PolicyDocument> = {
  terms: {
    id: 'terms',
    title: 'Terms of Service',
    lastUpdated: 'September 2026',
    content: `# Agora Terms of Service

*Last updated: September 2026*

Welcome to **Agora**, the decentralized event ticketing platform powered by the Stellar network and Soroban smart contracts. By accessing or using Agora, you agree to be bound by these Terms of Service.

---

### 1. Overview & Platform Role
Agora provides a non-custodial decentralized marketplace connecting **Event Organizers** and **Ticket Purchasers**. Agora does not hold custody of ticket funds or escrow private keys. All purchases and secondary transfers are executed directly on the **Stellar blockchain**.

### 2. User Responsibilities
When interacting with Agora, you agree to:
- Maintain the confidentiality and security of your private keys and seed phrases.
- Ensure all wallet transactions and smart contract calls are reviewed before approval.
- Refrain from unauthorized automated scraping, ticket scalping bot operations, or denial-of-service activities.

### 3. Ticket Purchases & Smart Contract Execution
- **Atomic Settlement:** Tickets are issued as cryptographic tokens on Stellar. Purchases are final upon transaction confirmation.
- **Gas & Network Fees:** Network transaction fees (in XLM) are paid directly to the Stellar network validators.
- **Refund Policy:** Organizer refund guarantees are enforced according to the specific event's smart contract parameters.

> **Important Notice:** Transactions on the blockchain cannot be reversed or modified once committed to a ledger. Always verify event details and recipient addresses.

### 4. Secondary Market & Resale Caps
Organizers may configure anti-scalping parameters, including:
1. Maximum resale price caps above initial face value.
2. Automated royalty redistribution to original event creators.
3. Geo-fenced or timed access restrictions.

### 5. Limitation of Liability
To the maximum extent permitted by applicable law, Agora is provided *"as is"* and *"as available"*, without warranties of any kind. Agora is not liable for blockchain reorganizations, wallet provider malfunctions, or organizer cancellation disputes.

---

For legal inquiries or dispute resolution, contact: [legal@agora-events.org](https://agora-events.org/legal)
`,
  },
  privacy: {
    id: 'privacy',
    title: 'Privacy Policy',
    lastUpdated: 'September 2026',
    content: `# Agora Privacy Policy

*Last updated: September 2026*

At **Agora**, we prioritize privacy, data minimization, and user sovereignty in accordance with web3 principles.

---

### 1. Data We Do NOT Collect
- We **never** collect, transmit, or store your private keys or seed phrases.
- We do not sell personal information to third-party advertising networks.
- We do not perform persistent biometric profiling.

### 2. Information We Process
We process minimal operational data strictly necessary to deliver the service:
- **Public Wallet Address:** Used to query ticket ownership and sign Soroban contract invocations.
- **Event Attendance Records:** Cryptographic proofs verified during on-site gate check-in.
- **Device Telemetry:** Anonymous crash logs and connection status diagnostics to improve mobile client performance.

### 3. On-Chain Transparency
Please note that transactions broadcast to the **Stellar Network** are public and permanently recorded on a decentralized distributed ledger:
1. Public addresses of ticket buyers and sellers.
2. Token minting, transfer, and redemption timestamps.
3. Contract invocation arguments and event emissions.

> **Privacy Recommendation:** We recommend utilizing dedicated event attendance wallets separate from high-value storage accounts.

### 4. User Rights & Data Deletion
Depending on your jurisdiction (such as GDPR or CCPA), you may have the right to request deletion of any off-chain cached metadata. Contact us at [privacy@agora-events.org](https://agora-events.org/privacy).
`,
  },
  refunds: {
    id: 'refunds',
    title: 'Ticket & Refund Policy',
    lastUpdated: 'September 2026',
    content: `# Ticket & Refund Policy

*Last updated: September 2026*

This policy outlines how cancellations, event rescheduling, and ticket refunds are handled across the **Agora** platform.

---

### 1. Event Cancellation
If an event organizer cancels an event:
- Automated smart contract escrow release triggers where applicable.
- Ticket holders receive a refund of the base purchase price back to their original purchasing wallet address.
- Network gas fees consumed during initial minting are non-refundable.

### 2. Rescheduling
If an event is rescheduled:
- Tickets remain automatically valid for the rescheduled date.
- Organizers may offer a 14-day refund claim window via the event smart contract.

### 3. Peer-to-Peer Resale
- Tickets purchased on the secondary market are subject to the same smart contract terms.
- Resale prices are capped according to the organizer's anti-scalping rules.

> For questions regarding specific event refunds, check the event page in the Agora app.
`,
  },
};

/**
 * Basic Markdown Style Renderer
 * Formats headers, bold, italics, lists, quotes, code, and links cleanly.
 */
function MarkdownRenderer({ content, colorScheme }: { content: string; colorScheme: 'light' | 'dark' }) {
  const isDark = colorScheme === 'dark';
  const textColor = isDark ? '#FFFFFF' : '#111827';
  const subtextColor = isDark ? '#9CA3AF' : '#4B5563';
  const borderColor = isDark ? '#2C2C2E' : '#E5E7EB';
  const quoteBg = isDark ? '#1C1917' : '#F9FAFB';
  const codeBg = isDark ? '#18181B' : '#F3F4F6';
  const accentColor = Colors.primaryYellow;

  const lines = useMemo(() => content.split('\n'), [content]);

  const renderInlineFormattedText = (text: string, baseStyle: any) => {
    // Regex matches: [link](url), **bold**, *italic*, `code`
    const tokens = [];
    const pattern = /(\[[^\]]+\]\([^)]+\)|\*\*\*[^*]+\*\*\*|\*\*[^*]+\*\*|\*[^*]+\*|`[^`]+`)/g;
    let lastIndex = 0;
    let match;

    while ((match = pattern.exec(text)) !== null) {
      if (match.index > lastIndex) {
        tokens.push({
          type: 'text',
          value: text.substring(lastIndex, match.index),
        });
      }

      const matchText = match[0];
      if (matchText.startsWith('[') && matchText.includes('](')) {
        const linkLabel = matchText.substring(1, matchText.indexOf(']('));
        const linkUrl = matchText.substring(matchText.indexOf('](') + 2, matchText.length - 1);
        tokens.push({ type: 'link', label: linkLabel, url: linkUrl });
      } else if (matchText.startsWith('***') && matchText.endsWith('***')) {
        tokens.push({ type: 'bold_italic', value: matchText.slice(3, -3) });
      } else if (matchText.startsWith('**') && matchText.endsWith('**')) {
        tokens.push({ type: 'bold', value: matchText.slice(2, -2) });
      } else if (matchText.startsWith('*') && matchText.endsWith('*')) {
        tokens.push({ type: 'italic', value: matchText.slice(1, -1) });
      } else if (matchText.startsWith('`') && matchText.endsWith('`')) {
        tokens.push({ type: 'code', value: matchText.slice(1, -1) });
      }
      lastIndex = pattern.lastIndex;
    }

    if (lastIndex < text.length) {
      tokens.push({
        type: 'text',
        value: text.substring(lastIndex),
      });
    }

    return (
      <Text style={baseStyle}>
        {tokens.map((token, index) => {
          if (token.type === 'bold') {
            return (
              <Text key={index} style={{ fontWeight: '700', color: textColor }}>
                {token.value}
              </Text>
            );
          }
          if (token.type === 'italic') {
            return (
              <Text key={index} style={{ fontStyle: 'italic' }}>
                {token.value}
              </Text>
            );
          }
          if (token.type === 'bold_italic') {
            return (
              <Text key={index} style={{ fontWeight: '700', fontStyle: 'italic', color: textColor }}>
                {token.value}
              </Text>
            );
          }
          if (token.type === 'code') {
            return (
              <Text
                key={index}
                style={{
                  fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
                  backgroundColor: codeBg,
                  color: isDark ? '#FBBF24' : '#B45309',
                  fontSize: 13,
                }}
              >
                {` ${token.value} `}
              </Text>
            );
          }
          if (token.type === 'link') {
            return (
              <Text
                key={index}
                style={{
                  color: isDark ? '#60A5FA' : '#2563EB',
                  textDecorationLine: 'underline',
                  fontWeight: '600',
                }}
                onPress={() => {
                  if (token.url) {
                    Linking.openURL(token.url).catch(() => {});
                  }
                }}
              >
                {token.label}
              </Text>
            );
          }
          return <React.Fragment key={index}>{token.value}</React.Fragment>;
        })}
      </Text>
    );
  };

  const renderedElements = [];
  let inCodeBlock = false;
  let codeBlockBuffer: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    const rawLine = lines[i];
    const trimmed = rawLine.trim();

    // Code block toggle
    if (trimmed.startsWith('```')) {
      if (inCodeBlock) {
        // End code block
        renderedElements.push(
          <View key={`code-${i}`} style={[styles.codeBlock, { backgroundColor: codeBg, borderColor }]}>
            <Text
              style={[
                styles.codeText,
                { color: isDark ? '#E5E7EB' : '#1F2937' },
              ]}
            >
              {codeBlockBuffer.join('\n')}
            </Text>
          </View>
        );
        codeBlockBuffer = [];
        inCodeBlock = false;
      } else {
        inCodeBlock = true;
      }
      continue;
    }

    if (inCodeBlock) {
      codeBlockBuffer.push(rawLine);
      continue;
    }

    // Empty line / paragraph break
    if (trimmed === '') {
      renderedElements.push(<View key={`spacer-${i}`} style={{ height: 10 }} />);
      continue;
    }

    // Horizontal rule
    if (trimmed === '---' || trimmed === '***' || trimmed === '___') {
      renderedElements.push(
        <View
          key={`hr-${i}`}
          style={[styles.hr, { borderBottomColor: borderColor }]}
        />
      );
      continue;
    }

    // Headers
    if (trimmed.startsWith('# ')) {
      renderedElements.push(
        <Text key={`h1-${i}`} style={[styles.h1, { color: textColor }]}>
          {trimmed.substring(2)}
        </Text>
      );
      continue;
    }
    if (trimmed.startsWith('## ')) {
      renderedElements.push(
        <Text key={`h2-${i}`} style={[styles.h2, { color: textColor }]}>
          {trimmed.substring(3)}
        </Text>
      );
      continue;
    }
    if (trimmed.startsWith('### ')) {
      renderedElements.push(
        <Text key={`h3-${i}`} style={[styles.h3, { color: textColor }]}>
          {trimmed.substring(4)}
        </Text>
      );
      continue;
    }
    if (trimmed.startsWith('#### ')) {
      renderedElements.push(
        <Text key={`h4-${i}`} style={[styles.h4, { color: textColor }]}>
          {trimmed.substring(5)}
        </Text>
      );
      continue;
    }

    // Blockquote
    if (trimmed.startsWith('> ')) {
      renderedElements.push(
        <View
          key={`quote-${i}`}
          style={[
            styles.blockquote,
            { backgroundColor: quoteBg, borderLeftColor: accentColor },
          ]}
        >
          {renderInlineFormattedText(trimmed.substring(2), [
            styles.quoteText,
            { color: subtextColor },
          ])}
        </View>
      );
      continue;
    }

    // Unordered List (- or *)
    if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
      renderedElements.push(
        <View key={`ul-${i}`} style={styles.listItem}>
          <Text style={[styles.bullet, { color: accentColor }]}>•</Text>
          <View style={styles.listTextContainer}>
            {renderInlineFormattedText(trimmed.substring(2), [
              styles.bodyText,
              { color: textColor },
            ])}
          </View>
        </View>
      );
      continue;
    }

    // Ordered List (1. 2. etc)
    const olMatch = trimmed.match(/^(\d+)\.\s+(.*)/);
    if (olMatch) {
      renderedElements.push(
        <View key={`ol-${i}`} style={styles.listItem}>
          <Text style={[styles.olNumber, { color: subtextColor }]}>
            {olMatch[1]}.
          </Text>
          <View style={styles.listTextContainer}>
            {renderInlineFormattedText(olMatch[2], [
              styles.bodyText,
              { color: textColor },
            ])}
          </View>
        </View>
      );
      continue;
    }

    // Default regular paragraph
    renderedElements.push(
      <View key={`p-${i}`} style={styles.paragraph}>
        {renderInlineFormattedText(rawLine, [
          styles.bodyText,
          { color: textColor },
        ])}
      </View>
    );
  }

  return <View style={styles.contentContainer}>{renderedElements}</View>;
}

export default function DocumentViewerScreen() {
  const router = useRouter();
  const params = useLocalSearchParams<{
    slug?: string;
    title?: string;
    content?: string;
  }>();
  const { colorScheme } = useThemeContext();
  const isDark = colorScheme === 'dark';

  const [activeSlug, setActiveSlug] = useState<string>(
    params.slug && DEFAULT_DOCUMENTS[params.slug] ? params.slug : 'terms'
  );

  const currentDoc: PolicyDocument = useMemo(() => {
    if (params.content) {
      return {
        id: 'custom',
        title: params.title || 'Document',
        lastUpdated: 'Current',
        content: params.content,
      };
    }
    return (
      DEFAULT_DOCUMENTS[activeSlug] ||
      DEFAULT_DOCUMENTS.terms
    );
  }, [params.content, params.title, activeSlug]);

  const backgroundColor = Colors[colorScheme].background;
  const textColor = isDark ? '#FFFFFF' : '#111827';
  const cardBg = Colors[colorScheme].cardBackground;
  const borderCol = Colors[colorScheme].border;

  return (
    <View style={[styles.screen, { backgroundColor }]}>
      {/* Pill tabs for switching between default policy documents */}
      {!params.content && (
        <View style={[styles.tabBar, { borderBottomColor: borderCol }]}>
          {Object.values(DEFAULT_DOCUMENTS).map((doc) => {
            const isSelected = activeSlug === doc.id;
            return (
              <TouchableOpacity
                key={doc.id}
                onPress={() => setActiveSlug(doc.id)}
                style={[
                  styles.tabPill,
                  {
                    backgroundColor: isSelected
                      ? Colors.primaryYellow
                      : cardBg,
                    borderColor: isSelected
                      ? Colors.primaryYellow
                      : borderCol,
                  },
                ]}
                accessibilityRole="button"
                accessibilityState={{ selected: isSelected }}
                accessibilityLabel={doc.title}
              >
                <Text
                  style={[
                    styles.tabPillText,
                    {
                      color: isSelected
                        ? '#000000'
                        : isDark
                        ? '#9CA3AF'
                        : '#4B5563',
                      fontWeight: isSelected ? '700' : '500',
                    },
                  ]}
                >
                  {doc.title}
                </Text>
              </TouchableOpacity>
            );
          })}
        </View>
      )}

      {/* Main ScrollView with smooth scrolling */}
      <ScrollView
        style={styles.scrollView}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={true}
        scrollEventThrottle={16}
        testID="document-scroll-view"
      >
        <View style={styles.headerBadgeRow}>
          <View style={[styles.statusBadge, { backgroundColor: cardBg }]}>
            <Ionicons name="document-text-outline" size={14} color={Colors.primaryYellow} />
            <Text style={[styles.statusBadgeText, { color: isDark ? '#9CA3AF' : '#6B7280' }]}>
              {currentDoc.lastUpdated}
            </Text>
          </View>
        </View>

        <MarkdownRenderer
          content={currentDoc.content}
          colorScheme={colorScheme}
        />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  screen: {
    flex: 1,
  },
  tabBar: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    paddingVertical: 10,
    gap: 8,
    borderBottomWidth: 1,
  },
  tabPill: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 20,
    borderWidth: 1,
  },
  tabPillText: {
    fontSize: 12,
  },
  scrollView: {
    flex: 1,
  },
  scrollContent: {
    paddingHorizontal: 20,
    paddingVertical: 18,
    paddingBottom: 40,
  },
  headerBadgeRow: {
    flexDirection: 'row',
    marginBottom: 12,
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 5,
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusBadgeText: {
    fontSize: 12,
    fontWeight: '500',
  },
  contentContainer: {
    width: '100%',
  },
  h1: {
    fontSize: 26,
    fontWeight: '800',
    lineHeight: 32,
    marginTop: 8,
    marginBottom: 8,
  },
  h2: {
    fontSize: 20,
    fontWeight: '700',
    lineHeight: 26,
    marginTop: 18,
    marginBottom: 8,
  },
  h3: {
    fontSize: 17,
    fontWeight: '700',
    lineHeight: 22,
    marginTop: 14,
    marginBottom: 6,
  },
  h4: {
    fontSize: 15,
    fontWeight: '600',
    lineHeight: 20,
    marginTop: 10,
    marginBottom: 4,
  },
  paragraph: {
    marginVertical: 4,
  },
  bodyText: {
    fontSize: 15,
    lineHeight: 22,
  },
  hr: {
    borderBottomWidth: 1,
    marginVertical: 16,
  },
  blockquote: {
    borderLeftWidth: 4,
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 4,
    marginVertical: 8,
  },
  quoteText: {
    fontSize: 14,
    lineHeight: 20,
    fontStyle: 'italic',
  },
  listItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginVertical: 3,
    paddingLeft: 4,
  },
  bullet: {
    fontSize: 18,
    lineHeight: 22,
    marginRight: 8,
  },
  olNumber: {
    fontSize: 14,
    fontWeight: '600',
    lineHeight: 22,
    marginRight: 8,
    minWidth: 20,
  },
  listTextContainer: {
    flex: 1,
  },
  codeBlock: {
    borderWidth: 1,
    borderRadius: 8,
    padding: 12,
    marginVertical: 8,
  },
  codeText: {
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
    fontSize: 13,
    lineHeight: 18,
  },
});
