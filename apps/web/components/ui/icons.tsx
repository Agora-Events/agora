interface IconProps {
  size?: number;
  className?: string;
}

function Icon({ src, alt, size = 24, className }: IconProps & { src: string; alt: string }) {
  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img src={src} alt={alt} width={size} height={size} className={className} />
  );
}

export function ChevronDown(props: IconProps) {
  return <Icon src="/icons/chevron-down.svg" alt="chevron down" {...props} />;
}

export function ChevronUp(props: IconProps) {
  return <Icon src="/icons/chevron-up.svg" alt="chevron up" {...props} />;
}

export function Camera(props: IconProps) {
  return <Icon src="/icons/camera.svg" alt="camera" {...props} />;
}

export function CheckCircle2(props: IconProps) {
  return <Icon src="/icons/check-circle.svg" alt="check circle" {...props} />;
}

export function Home(props: IconProps) {
  return <Icon src="/icons/home.svg" alt="home" {...props} />;
}

export function ExternalLink(props: IconProps) {
  return <Icon src="/icons/external-link.svg" alt="external link" {...props} />;
}

export function X(props: IconProps) {
  return <Icon src="/icons/close.svg" alt="close" {...props} />;
}

export function Minus(props: IconProps) {
  return <Icon src="/icons/minus.svg" alt="minus" {...props} />;
}

export function Plus(props: IconProps) {
  return <Icon src="/icons/plus.svg" alt="plus" {...props} />;
}

export function Ticket(props: IconProps) {
  return <Icon src="/icons/ticket.svg" alt="ticket" {...props} />;
}

export function ArrowRight(props: IconProps) {
  return <Icon src="/icons/arrow-right.svg" alt="arrow right" {...props} />;
}

export function Gift(props: IconProps) {
  return <Icon src="/icons/gift.svg" alt="gift" {...props} />;
}
