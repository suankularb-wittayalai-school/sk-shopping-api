// External libraries
import Image from "next/image";
import Head from "next/head";

import { useTranslation } from "next-i18next";
import { serverSideTranslations } from "next-i18next/serverSideTranslations";

// SK Components
import {
  ContentLayout,
  Header,
  MaterialIcon,
  Section,
} from "@suankularb-components/react";

// Types
import { CustomPage, LangCode } from "@/utils/types";

// Page
const IndexPage: CustomPage = () => {
  return (
    <>
      <Head>
        <title>SK Shopping Test Server</title>
      </Head>
      <ContentLayout>
        <Section>
          {/* <Header>{t("welcome.title")}</Header>
          <p className="skc-body-medium">{t("welcome.desc")}</p> */}
        </Section>
      </ContentLayout>
    </>
  );
};

export const getStaticProps = async ({ locale }: { locale: LangCode }) => ({
  props: {
    ...(await serverSideTranslations(locale, ["common", "home"])),
  },
});

// IndexPage.pageHeader = {
//   title: { key: "title", ns: "home" },
//   // icon: <MaterialIcon icon="waving_hand" />,
// };

export default IndexPage;
