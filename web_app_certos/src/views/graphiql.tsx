
import { useEffect } from "react";
import { useSafeSetState } from "react-admin";
import { Box } from "@mui/material";
import { GraphiQL } from "graphiql";
import { createGraphiQLFetcher } from '@graphiql/toolkit';
import { getRawAuthorization } from "../components/auth_provider";
import type { GraphQLSchema } from "graphql";
import { envs } from "../components/cypher";
import Loading from "./loading";
import 'graphiql/graphiql.css';


const Graphiql = () => {
  const [schema, setSchema] = useSafeSetState<GraphQLSchema>();
  const [body, setBody] = useSafeSetState();
  const [headers, setHeaders] = useSafeSetState("{}");
  const origin = envs[localStorage.getItem("environment")].url;
  const graphqlUrl = `${origin}/graphql`;
  const fetcher = createGraphiQLFetcher({url: graphqlUrl});


  useEffect(() => {
    const init = async () => {
      const response = await fetch(`${origin}/graphql/introspect`);
      setSchema(await response.json());
    }
    init();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);


  const change = async (s) => {
    const maybeNewBody = JSON.stringify({query: s.tabs[s.activeTabIndex].query});
    if (body !== maybeNewBody) {
      setHeaders(JSON.stringify({Authentication: await getRawAuthorization(graphqlUrl, "POST", maybeNewBody)}));
      setBody(maybeNewBody);
    }
  }

  if (!schema) return <Loading />;

  return(
    <Box sx={{height: "100vh"}}>
      <GraphiQL
        fetcher={fetcher}
        headers={headers}
        schema={schema}
        onTabChange={change}
      />
    </Box>
  )
}

export default Graphiql;