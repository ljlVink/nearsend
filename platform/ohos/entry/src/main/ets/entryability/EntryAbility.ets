import { AbilityConstant, Want } from "@kit.AbilityKit";
import pasteboard from "@ohos.pasteboard";
import { RustAbility } from "@ohos-rs/ability";
import nearSend from "libnear_send.so";

export default class EntryAbility extends RustAbility {
  public moduleName: string = "near_send";

  private async readClipboardText(): Promise<string> {
    try {
      const systemPasteboard = pasteboard.getSystemPasteboard();
      const data = await systemPasteboard.getData();
      const text = data?.getPrimaryText();
      return text ? String(text) : "";
    } catch (_err) {
      return "";
    }
  }

  private async writeClipboardText(text: string): Promise<boolean> {
    try {
      const systemPasteboard = pasteboard.getSystemPasteboard();
      const data = pasteboard.createData(pasteboard.MIMETYPE_TEXT_PLAIN, text);
      await systemPasteboard.setData(data);
      return true;
    } catch (_err) {
      return false;
    }
  }

  public async onCreate(
    want: Want,
    launchParam: AbilityConstant.LaunchParam,
  ): Promise<void> {
    const registerFn =
      nearSend?.registerClipboardCallbacks;
    if (typeof registerFn === "function") {
      registerFn(
        async () => await this.readClipboardText(),
        async (text: string) => await this.writeClipboardText(text),
      );
    }
    await super.onCreate(want, launchParam);
  }
}
