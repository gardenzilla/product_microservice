Images
---

service SkuImages {
  rpc New(NewRequest) returns (e);
  rpc GetImages(e) returns (e);
}

message NewRequest {
  uint32 sku = 1;
  bytes image_bytes = 2;
  string file_name = 3;
  string file_ext = 4;
}

image ================> SKU ============> SkuImageProcesser =======> SkuImageStatic
        Store Image      |    Process IMAGE        *          Store images
          to SKU         |      CROP and RESIZE   /|\           in static storage
                         |        & send to Static |
                         *--------------------------
                            Rename image
                            & store its ID to SKU
