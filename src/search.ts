import { ListItem, Plugin } from "utools-helper";

export let activeSearch = "";

export abstract class SearchPlugin implements Plugin {
  abstract code: string;
  abstract search(word: string): Promise<ListItem[]>;

  protected get keyword(): string {
    return utools.dbStorage.getItem(`search:${this.code}`) || "";
  }

  protected remember(word: string) {
    activeSearch = this.code;
    utools.dbStorage.setItem(`search:${this.code}`, word);
  }

  async enter(): Promise<ListItem[]> {
    const word = this.keyword;
    utools.setSubInputValue(word);
    return this.search(word);
  }
}
