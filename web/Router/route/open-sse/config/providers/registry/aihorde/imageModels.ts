/**
 * AI Horde image-generation provider entry.
 *
 * Chat still goes through oai.aihorde.net. Image jobs use the native Horde
 * async API (`/v2/generate/async`). `models` is a live getter so
 * imageRegistry stays under the file-size cap and zero-worker names are
 * never advertised.
 *
 * NOTE: the original design received live models through a
 * `services/dynamicImageModelSources.ts` module that was never copied over
 * in the OmniRoute merge (nothing in the repo provides
 * `getDynamicImageModels`). The import is replaced with a local stub
 * returning no models until that catalog service is ported — an explicit
 * empty list is safer than a dangling import that breaks the whole
 * provider registry at load time.
 */

interface ImageModelStub {
  id: string;
  name: string;
}

function getDynamicImageModels(_provider: string): ImageModelStub[] {
  return [];
}

export const AI_HORDE_IMAGE_PROVIDER = {
  id: "aihorde",
  alias: "horde",
  baseUrl: "https://aihorde.net/api",
  authType: "apikey",
  authHeader: "apikey",
  format: "aihorde",
  get models() {
    return getDynamicImageModels("aihorde");
  },
  supportedSizes: ["512x512", "768x768", "1024x1024", "1024x768", "768x1024"],
};
