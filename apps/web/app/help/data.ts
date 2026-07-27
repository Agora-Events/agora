import fs from 'fs';
import path from 'path';

export interface Article {
  slug: string;
  title: string;
  categorySlug: string;
  content: string;
}

export const mockArticles: Article[] = [
  {
    slug: "what-is-agora",
    title: "What is Agora?",
    categorySlug: "getting-started",
    content: "# What is Agora?\n\nAgora is a decentralized event management platform...\n\n## Getting Started\n\n1. Sign up for an account\n2. Create an event\n3. Sell tickets",
  },
  {
    slug: "how-to-buy-tickets",
    title: "How to Buy Tickets",
    categorySlug: "buying-tickets",
    content: "# How to Buy Tickets\n\nBuying tickets on Agora is simple.\n\n## Steps\n\n- Navigate to an event page\n- Click **Buy Ticket**\n- Confirm transaction in your wallet",
  }
];

// Helper to extract title from markdown
const extractTitle = (content: string): string => {
  const match = content.match(/^#\s+(.+)$/m);
  return match ? match[1].trim() : "Untitled Document";
};

// Function to fetch articles combining mock and dynamically loaded MDX files
const fetchAllArticles = (): Article[] => {
  const articles: Article[] = [...mockArticles];
  
  try {
    const contentDir = path.join(process.cwd(), 'content', 'help');
    
    // We only have stellar-web3 MDX files right now
    if (fs.existsSync(contentDir)) {
      const files = fs.readdirSync(contentDir);
      
      files.forEach((filename) => {
        if (filename.endsWith('.md') || filename.endsWith('.mdx')) {
          const filePath = path.join(contentDir, filename);
          const content = fs.readFileSync(filePath, 'utf8');
          const slug = filename.replace(/\.mdx?$/, '');
          const title = extractTitle(content);
          
          articles.push({
            slug,
            title,
            categorySlug: 'stellar-web3',
            content
          });
        }
      });
    }
  } catch (error) {
    console.error("Error reading help content directory:", error);
  }

  return articles;
};

export const getArticlesByCategory = (categorySlug: string): Article[] => {
  const allArticles = fetchAllArticles();
  return allArticles.filter((a) => a.categorySlug === categorySlug);
};

export const getArticle = (categorySlug: string, slug: string): Article | undefined => {
  const allArticles = fetchAllArticles();
  return allArticles.find((a) => a.categorySlug === categorySlug && a.slug === slug);
};
