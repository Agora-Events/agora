import type { Metadata } from "next";
import { notFound } from "next/navigation";
import Link from "next/link";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { Breadcrumb } from "@/components/ui/breadcrumb";
import { getArticle, getArticlesByCategory } from "../../data";
import { ClientSidebar } from "./client-sidebar";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ category: string; slug: string }>;
}): Promise<Metadata> {
  const { category, slug } = await params;
  const article = getArticle(category, slug);

  if (!article) {
    return {
      title: "Article Not Found | Agora Help Center",
    };
  }

  const title = `${article.title} | Help Center - Agora`;
  const description = article.summary || article.content.slice(0, 160).replace(/[#*`]/g, "").trim();
  const canonicalUrl = `https://agora.events/help/${category}/${slug}`;

  return {
    title,
    description,
    alternates: {
      canonical: canonicalUrl,
    },
    openGraph: {
      title,
      description,
      url: canonicalUrl,
      type: "article",
      siteName: "Agora Events",
    },
    twitter: {
      card: "summary_large_image",
      title,
      description,
    },
  };
}

export default async function HelpArticlePage({
  params,
}: {
  params: Promise<{ category: string; slug: string }>;
}) {
  const { category, slug } = await params;
  
  const article = getArticle(category, slug);
  if (!article) {
    notFound();
  }

  const categoryArticles = getArticlesByCategory(category);

  // "getting-started" -> "Getting Started"
  const categoryLabel = category
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");

  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />

      {/* Breadcrumb & Header Banner */}
      <div className="bg-[#FDDA23] border-b-2 border-black pt-28 pb-8 px-4 md:px-8 shadow-[0px_4px_0px_0px_rgba(0,0,0,1)]">
        <div className="max-w-5xl mx-auto w-full flex flex-col gap-4">
          <Breadcrumb
            items={[
              { label: "Home", href: "/" },
              { label: "Help", href: "/help" },
              // No /help/[category] index route exists yet, so this level is
              // context only rather than a link to a 404.
              { label: categoryLabel },
              { label: article.title },
            ]}
            className="font-semibold"
            linkClassName="text-black/70 hover:text-black"
            separatorClassName="text-black/40"
            currentClassName="text-black"
          />
          <h1 className="text-3xl md:text-5xl font-black tracking-tight text-black">
            {article.title}
          </h1>
        </div>
      </div>

      <div className="flex-1 w-full max-w-5xl mx-auto px-4 md:px-8 py-12 flex flex-col md:flex-row gap-8 lg:gap-12">
        {/* Sidebar */}
        <ClientSidebar 
          categorySlug={category} 
          currentSlug={slug} 
          articles={categoryArticles} 
        />

        {/* Article Content */}
        <article className="flex-1 bg-white border-2 border-black rounded-3xl p-6 md:p-10 shadow-[-6px_6px_0px_0px_rgba(0,0,0,1)]">
          <div className="prose prose-lg prose-yellow max-w-none 
                          prose-headings:font-black prose-headings:text-black 
                          prose-p:text-gray-700 prose-p:leading-relaxed 
                          prose-a:text-black prose-a:font-bold prose-a:underline prose-a:decoration-2 prose-a:decoration-[#FDDA23] hover:prose-a:text-gray-800
                          prose-strong:text-black prose-strong:font-bold
                          prose-code:text-black prose-code:bg-[#FDDA23] prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded-md prose-code:font-bold
                          prose-pre:bg-black prose-pre:text-white prose-pre:border-2 prose-pre:border-black prose-pre:shadow-[-4px_4px_0px_0px_rgba(253,218,35,1)]
                          prose-li:text-gray-700">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {article.content}
            </ReactMarkdown>
          </div>
        </article>
      </div>

      {/* Footer Contact Banner */}
      <section className="px-4 md:px-8 pb-24 max-w-5xl mx-auto w-full mt-12">
        <div className="border-2 border-black rounded-2xl bg-[#FFFBE9] p-8 shadow-[-6px_6px_0px_0px_rgba(0,0,0,1)] flex flex-col sm:flex-row items-start sm:items-center justify-between gap-6">
          <div>
            <p className="font-black text-lg text-black">
              Didn&apos;t find your answer?
            </p>
            <p className="text-sm text-gray-600 mt-1">
              Our support team is always here to help you.
            </p>
          </div>
          <Link
            href="/contact"
            className="shrink-0 inline-flex items-center gap-2 px-6 py-3 rounded-full border-2 border-black bg-[#FDDA23] text-black text-sm font-bold
                       shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)]
                       hover:shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]
                       hover:-translate-x-[1px] hover:translate-y-[1px]
                       active:shadow-none active:-translate-x-[3px] active:translate-y-[3px]
                       transition-all"
          >
            Contact Support →
          </Link>
        </div>
      </section>

      <Footer />
    </main>
  );
}
