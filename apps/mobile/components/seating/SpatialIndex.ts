import { SeatData, ViewportBounds } from './types';

/**
 * QuadTree Spatial Index for Frustum Culling
 * Enables instantaneous sub-millisecond range queries across 10,000+ seats.
 */
export class QuadTreeNode {
  bounds: ViewportBounds;
  capacity: number;
  seats: SeatData[] = [];
  divided: boolean = false;

  northwest?: QuadTreeNode;
  northeast?: QuadTreeNode;
  southwest?: QuadTreeNode;
  southeast?: QuadTreeNode;

  constructor(bounds: ViewportBounds, capacity: number = 32) {
    this.bounds = bounds;
    this.capacity = capacity;
  }

  private subdivide(): void {
    const { minX, minY, maxX, maxY } = this.bounds;
    const midX = (minX + maxX) / 2;
    const midY = (minY + maxY) / 2;

    this.northwest = new QuadTreeNode({ minX, minY, maxX: midX, maxY: midY }, this.capacity);
    this.northeast = new QuadTreeNode({ minX: midX, minY, maxX, maxY: midY }, this.capacity);
    this.southwest = new QuadTreeNode({ minX, minY: midY, maxX: midX, maxY }, this.capacity);
    this.southeast = new QuadTreeNode({ minX: midX, minY: midY, maxX, maxY }, this.capacity);

    this.divided = true;

    // Distribute existing seats to children
    const existing = this.seats;
    this.seats = [];
    for (const seat of existing) {
      this.insert(seat);
    }
  }

  private contains(bounds: ViewportBounds, x: number, y: number): boolean {
    return x >= bounds.minX && x <= bounds.maxX && y >= bounds.minY && y <= bounds.maxY;
  }

  private intersects(a: ViewportBounds, b: ViewportBounds): boolean {
    return !(
      a.maxX < b.minX ||
      a.minX > b.maxX ||
      a.maxY < b.minY ||
      a.minY > b.maxY
    );
  }

  insert(seat: SeatData): boolean {
    if (!this.contains(this.bounds, seat.x, seat.y)) {
      return false;
    }

    if (!this.divided) {
      if (this.seats.length < this.capacity) {
        this.seats.push(seat);
        return true;
      }
      this.subdivide();
    }

    if (this.northwest?.insert(seat)) return true;
    if (this.northeast?.insert(seat)) return true;
    if (this.southwest?.insert(seat)) return true;
    if (this.southeast?.insert(seat)) return true;

    return false;
  }

  query(viewport: ViewportBounds, found: SeatData[] = []): SeatData[] {
    if (!this.intersects(this.bounds, viewport)) {
      return found;
    }

    for (const seat of this.seats) {
      if (this.contains(viewport, seat.x, seat.y)) {
        found.push(seat);
      }
    }

    if (this.divided) {
      this.northwest?.query(viewport, found);
      this.northeast?.query(viewport, found);
      this.southwest?.query(viewport, found);
      this.southeast?.query(viewport, found);
    }

    return found;
  }

  clear(): void {
    this.seats = [];
    this.divided = false;
    this.northwest = undefined;
    this.northeast = undefined;
    this.southwest = undefined;
    this.southeast = undefined;
  }
}

export class QuadTreeIndex {
  root: QuadTreeNode;

  constructor(bounds: ViewportBounds, capacity: number = 32) {
    this.root = new QuadTreeNode(bounds, capacity);
  }

  build(seats: SeatData[]): void {
    this.root.clear();
    for (let i = 0; i < seats.length; i++) {
      this.root.insert(seats[i]);
    }
  }

  queryVisibleSeats(viewport: ViewportBounds, marginMultiplier: number = 0.2): SeatData[] {
    // Expand viewport slightly with a frustum margin to avoid edge pop-in
    const width = viewport.maxX - viewport.minX;
    const height = viewport.maxY - viewport.minY;
    const marginX = width * marginMultiplier;
    const marginY = height * marginMultiplier;

    const expandedBounds: ViewportBounds = {
      minX: viewport.minX - marginX,
      minY: viewport.minY - marginY,
      maxX: viewport.maxX + marginX,
      maxY: viewport.maxY + marginY,
    };

    return this.root.query(expandedBounds);
  }
}
