// A component that fetches data from the server and displays it as formatted json

import { fetchAPI } from "@/utils/backend";
import {
  Actions,
  Button,
  Columns,
  FormGroup,
  FormItem,
  Header,
  MenuItem,
  Section,
  Select,
  TextField,
} from "@suankularb-components/react";
import { useState } from "react";

export default function FetchComponent({
  accessToken,
}: {
  accessToken?: string;
}) {
  const [path, setPath] = useState<string>("");
  const [method, setMethod] = useState<
    "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
  >("GET");

  const [returnResponse, setReturnResponse] = useState<string>("");

  const [loading, setLoading] = useState<boolean>(false);

  async function sendRequest() {
    setLoading(true);
    const response = await fetchAPI(
      path,
      undefined,
      {
        method,
        headers: {
          "Content-Type": "application/json",
        },
      },
      accessToken
    );
    setReturnResponse(JSON.stringify(response, null, 2));
    setLoading(false);
  }

  // render a text box to enter path
  // render a dropdown to select method
  // render a text box to enter body (if method is POST or PUT or PATCH)
  // render a button to send request
  // render a text box to display response as formatted json
  return (
    <>
      <Section>
        <Header>API Fetcher</Header>
        <Columns columns={4}>
          <TextField<string>
            appearance="outlined"
            label="Path"
            behavior="single-line"
            helperMsg="Enter path"
            value={path}
            onChange={setPath}
            inputAttr={{ autoCorrect: "off", autoCapitalize: "none" }}
            className="col-span-3"
          />

          <Select
            appearance="outlined"
            label="Method"
            helperMsg="Select method"
            value={method}
            onChange={setMethod}
          >
            <MenuItem value="GET">GET</MenuItem>
            <MenuItem value="POST">POST</MenuItem>
            <MenuItem value="PUT">PUT</MenuItem>
            <MenuItem value="PATCH">PATCH</MenuItem>
            <MenuItem value="DELETE">DELETE</MenuItem>
          </Select>
        </Columns>
        <Actions>
          <Button
            onClick={sendRequest}
            appearance="filled"
            className="!margin-12"
          >
            Send Request
          </Button>
        </Actions>
      </Section>
      <Section>
        {loading && <p>Loading...</p>}
        {!loading && <pre>{returnResponse}</pre>}
      </Section>
    </>
  );
}
