import { NextRequest, NextResponse } from "next/server";
import { getAuthFromRequest } from "@/lib/auth";

const MAX_UPLOAD_BYTES = 5 * 1024 * 1024;
const ALLOWED_TYPES = new Set(["image/jpeg", "image/png", "image/webp"]);

export async function POST(request: NextRequest) {
  if (!getAuthFromRequest(request)?.sub) {
    return NextResponse.json({ message: "Authentication required" }, { status: 401 });
  }

  const pinataJwt = process.env.PINATA_JWT;
  if (!pinataJwt) {
    return NextResponse.json(
      { message: "Image uploads are not configured" },
      { status: 503 },
    );
  }

  let file: FormDataEntryValue | null;
  try {
    file = (await request.formData()).get("file");
  } catch {
    return NextResponse.json({ message: "Invalid multipart upload" }, { status: 400 });
  }
  if (!(file instanceof File)) {
    return NextResponse.json({ message: "An image file is required" }, { status: 400 });
  }
  if (!ALLOWED_TYPES.has(file.type)) {
    return NextResponse.json(
      { message: "Only JPEG, PNG, and WebP images are supported" },
      { status: 400 },
    );
  }
  if (file.size > MAX_UPLOAD_BYTES) {
    return NextResponse.json({ message: "Image must be 5 MB or smaller" }, { status: 400 });
  }

  const upload = new FormData();
  upload.append("file", file, file.name);

  try {
    const response = await fetch("https://api.pinata.cloud/pinning/pinFileToIPFS", {
      method: "POST",
      headers: { Authorization: `Bearer ${pinataJwt}` },
      body: upload,
      cache: "no-store",
    });
    const result = await response.json().catch(() => ({}));
    if (!response.ok || typeof result.IpfsHash !== "string") {
      console.error("Pinata upload failed:", response.status, result);
      return NextResponse.json(
        { message: "The image could not be uploaded to IPFS" },
        { status: 502 },
      );
    }

    const cid = result.IpfsHash as string;
    return NextResponse.json({
      cid,
      url: `https://gateway.pinata.cloud/ipfs/${cid}`,
    });
  } catch (error) {
    console.error("IPFS upload request failed:", error);
    return NextResponse.json({ message: "The IPFS service is unavailable" }, { status: 502 });
  }
}