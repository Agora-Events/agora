# Agora Frontend

Welcome to the **Agora** frontend codebase! We are excited to have you contribute. This guide will help you understand our components, style guidelines, and best practices to keep the codebase clean and consistent.

## 🎨 Design & Style Guidelines

### Fidelity to Figma

We maintain **strict adherence** to our Figma designs.

- **Colors, Padding, Margins**: Please follow the Figma file exactly. Values that do not match the brand guidelines (e.g., using a random hex code instead of our palette or approximated spacing) will **not be merged**.
- **Pixel Perfection**: We aim for high-quality, pixel-perfect implementation.

### Icons

- **Location**: All icons are located in `public/icons/`.
- **Usage**: Please use the existing assets. **Do not install external icon libraries** (like FontAwesome, HeroIcons package, etc.) unless explicitly discussed. We prefer using local SVGs to keep the bundle size small and the design consistent.

## 🧩 Components

### Button Component

We have a reusable custom `Button` component located in `components/ui/button.tsx`. Please use this for all buttons to ensure consistent behavior (hover effects, shadows) across the app.

**Props:**

- `backgroundColor`: Tailwind class (e.g., `bg-[#FDDA23]`, `bg-white`) or hex code. Defaults to `bg-white`.
- `textColor`: Tailwind class (e.g., `text-black`) or hex code. Defaults to `text-black`.
- `shadowColor`: Hex or RGBA string for the hard shadow (e.g., `rgba(0,0,0,1)`).
- `className`: Additional Tailwind classes for width, height, etc.

**Example Usage:**

```tsx
import { Button } from "@/components/ui/button";

// Yellow button with black text and black shadow
<Button
  backgroundColor="bg-[#FDDA23]"
  textColor="text-black"
  shadowColor="rgba(0,0,0,1)"
  className="w-[200px]"
>
  <span>Get Started</span>
</Button>;
```

### Layout Components

- **Navbar**: Located in `components/layout/navbar.tsx`.
- **Footer**: Located in `components/layout/footer.tsx`.
- **Component Catalog**: See `DOCS/COMPONENTS.md` for the reusable component library, folder map, and contribution rules.

These should be used in your page layouts (e.g., `app/page.tsx`) to maintain a consistent wrapper.

## 🚀 Getting Started

1. **Run Development Server**:
   ```bash
   pnpm dev
   ```
2. **Linting**:
   Before submitting a PR, please ensure your code is linted:
   ```bash
   pnpm lint
   ```

---

**Happy Coding!** We appreciate your help in making Agora amazing. If you have any questions, feel free to reach out!
