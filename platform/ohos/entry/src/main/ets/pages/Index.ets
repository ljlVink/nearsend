import { DefaultXComponent } from "@ohos-rs/ability";
import nearSend from "libnear_send.so";

@Entry
@Component
struct Index {
  onBackPress(): boolean | void {
    if (typeof nearSend.onBackPressIntercept === "function") {
      return nearSend.onBackPressIntercept();
    }
    return false;
  }

  build() {
    RelativeContainer() {
      DefaultXComponent();
    }
    .height("100%")
    .width("100%");
  }
}
