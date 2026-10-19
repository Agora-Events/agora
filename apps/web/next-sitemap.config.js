/** @type {import('next-sitemap').IConfig} */
module.exports = {
  siteUrl: process.env.NEXT_PUBLIC_SITE_URL || "https://agora-web-eta.vercel.app",
  generateRobotsTxt: true,
  additionalPaths: async () => [
    { loc: "/", changefreq: "daily", priority: 1.0 },
    { loc: "/discover", changefreq: "daily", priority: 0.9 },
    { loc: "/pricing", changefreq: "monthly", priority: 0.7 },
    { loc: "/faqs", changefreq: "monthly", priority: 0.6 },
  ],
};
