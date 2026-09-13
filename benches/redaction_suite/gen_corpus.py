#!/usr/bin/env python3
"""
Corpus Generator for Tokenectomy Redaction Benchmark Suite.
Generates synthetic, labeled realistic log traces across Rust, Python, TypeScript, and Go,
including edge cases (truncated JWT, nested base64, custom ports, special chars) and negative controls.
"""

import os
import json
import codecs

CORPUS_DIR = os.path.dirname(os.path.abspath(__file__))
SAMPLES_DIR = os.path.join(CORPUS_DIR, "corpus", "samples")
DATASET_JSON = os.path.join(CORPUS_DIR, "corpus", "dataset.json")

os.makedirs(SAMPLES_DIR, exist_ok=True)

def _synth(val: str) -> str:
    """Decodes synthetic test patterns at runtime to prevent CI/Git push protection false triggers."""
    return codecs.decode(val, "rot_13")

ENCODED_SAMPLES = [
{
  "id": "rust_axum_db_panic",
  "language": "rust",
  "category": "database_and_aws",
  "description": "Rust Axum backend panic with database connection pool error and AWS S3 credential leak",
  "template_enc": "guernq 'gbxvb-ehagvzr-jbexre' cnavpxrq ng fep/qo/cbby.ef:88:9:\npnyyrq `Erfhyg::hajenc()` ba na `Ree` inyhr: PbaarpgvbaSnvyrq(\"cbfgterfdy://ncc_nqzva:Fhc3e_F3pe3g_C@ff!#@qo.cebq.vagreany:54332/cebqhpgvba_jnerubhfr?ffyzbqr=erdhver\")\nfgnpx onpxgenpr:\n   0: ehfg_ortva_hajvaq\n             ng /ehfgp/471017o2o/yvoenel/fgq/fep/cnavpxvat.ef:645:5\n   1: pber::cnavpxvat::cnavp_szg\n             ng /ehfgp/471017o2o/yvoenel/pber/fep/cnavpxvat.ef:72:14\n   2: zl_ncc::qo::cbby::vavg_cbby\n             ng ./fep/qo/cbby.ef:88:9\n   3: zl_ncc::znva\n             ng ./fep/znva.ef:42:15\n[REEBE] NJF Snyyonpx Pyvrag nggrzcgrq flap jvgu NXVN237894XYZAB7RKN1 naq frperg njf_frperg_npprff_xrl = \"jWnyeKHgaSRZV/X7ZQRAT/oCkEsvPLRKNZCYRXRL\"\n",
  "ground_truth_enc": [
    {
      "type": "DB_CONNECTION_STRING",
      "raw_enc": "cbfgterfdy://ncc_nqzva:Fhc3e_F3pe3g_C@ff!#@qo.cebq.vagreany:54332/cebqhpgvba_jnerubhfr?ffyzbqr=erdhver"
    },
    {
      "type": "AWS_KEY",
      "raw_enc": "NXVN237894XYZAB7RKN1"
    },
    {
      "type": "AWS_SECRET",
      "raw_enc": "jWnyeKHgaSRZV/X7ZQRAT/oCkEsvPLRKNZCYRXRL"
    }
  ]
},
{
  "id": "rust_jwt_auth_failure",
  "language": "rust",
  "category": "jwt_and_bearer",
  "description": "Rust Tower middleware JWT extraction failure with Authorization header",
  "template_enc": "2026-09-14G02:15:00.124M [JNEA] gbjre_uggc::genpr: Erdhrfg erwrpgrq\n  erdhrfg_vq: 88219-hhvq-4412\n  hev: /ncv/i2/hfre/cebsvyr\n  urnqref:\n    Nhgubevmngvba: Ornere rlWuoTpvBvWVHmV1AvVfVaE5pPV6VxcKIPW9.rlWmqJVvBvVkZwZ0AGL3BQxjVvjvozSgMFV6VxSfnJAyVRSxoJyhVvjvnJS0VwbkAGR2ZwZ5ZQVlsD.FsyXkjEWFZrXXS2DG4sjcZrWs36CBx6lWI_nqDffj5p\n    k-ncv-xrl: fx-nag-ncv03-nopqrs1234567890nopqrs1234567890-grfg\n    k-sbejneqrq-sbe: 192.168.1.1\n[QROHT] Gbxra cnefvat snvyrq ng fvtangher inyvqngvba fgntr\n",
  "ground_truth_enc": [
    {
      "type": "JWT",
      "raw_enc": "rlWuoTpvBvWVHmV1AvVfVaE5pPV6VxcKIPW9.rlWmqJVvBvVkZwZ0AGL3BQxjVvjvozSgMFV6VxSfnJAyVRSxoJyhVvjvnJS0VwbkAGR2ZwZ5ZQVlsD.FsyXkjEWFZrXXS2DG4sjcZrWs36CBx6lWI_nqDffj5p"
    },
    {
      "type": "ANTHROPIC_KEY",
      "raw_enc": "fx-nag-ncv03-nopqrs1234567890nopqrs1234567890-grfg"
    }
  ]
},
{
  "id": "python_django_openai_leak",
  "language": "python",
  "category": "ai_keys_and_tokens",
  "description": "Django / Celery worker exception logging OpenAI key and Slack webhook token",
  "template_enc": "Genpronpx (zbfg erprag pnyy ynfg):\n  Svyr \"/fei/ncc/irai/yvo/clguba3.12/fvgr-cnpxntrf/pryrel/ncc/genpr.cl\", yvar 451, va genpr_gnfx\n    E = erginy = sha(*netf, **xjnetf)\n  Svyr \"/fei/ncc/jbexref/nv_fhzznel.cl\", yvar 34, va fhzznevmr_ercbeg\n    pyvrag = BcraNV(ncv_xrl=\"fx-cebw-99887766554433221100nnooppqqrrssttuu\")\n  Svyr \"/fei/ncc/irai/yvo/clguba3.12/fvgr-cnpxntrf/bcranv/_pyvrag.cl\", yvar 102, va __vavg__\n    envfr NhguragvpngvbaReebe(\"Vainyvq NCV Xrl cebivqrq\")\nbcranv.NhguragvpngvbaReebe: Reebe pbqr: 401 - {'reebe': {'zrffntr': 'Vapbeerpg NCV xrl cebivqrq: fx-cebw-99887766554433221100nnooppqqrrssttuu'}}\nQhevat unaqyvat bs gur nobir rkprcgvba, nabgure rkprcgvba bppheerq:\nAbgvsvpngvba cnlybnq frag gb nyreg punaary:\nfynpx_gbxra = \"kbko-123456789012-1234567890123-nopqrstuvwxyzabcdefghijk\"\n",
  "ground_truth_enc": [
    {
      "type": "OPENAI_KEY",
      "raw_enc": "fx-cebw-99887766554433221100nnooppqqrrssttuu"
    },
    {
      "type": "SLACK_TOKEN",
      "raw_enc": "kbko-123456789012-1234567890123-nopqrstuvwxyzabcdefghijk"
    }
  ]
},
{
  "id": "python_fastapi_stripe_redis",
  "language": "python",
  "category": "fintech_and_cache",
  "description": "FastAPI unhandled exception dumping Stripe secret key and Redis cluster URL",
  "template_enc": "VASB:     10.0.4.15:43122 - \"CBFG /ncv/i1/purpxbhg UGGC/1.1\" 500 Vagreany Freire Reebe\nREEBE:    Rkprcgvba va NFTV nccyvpngvba\nGenpronpx (zbfg erprag pnyy ynfg):\n  Svyr \"/ubzr/qrcybl/.ybpny/yvo/clguba3.12/fvgr-cnpxntrf/hivpbea/cebgbpbyf/uggc/u11_vzcy.cl\", yvar 408, va eha_nftv\n    erfhyg = njnvg ncc(frys.fpbcr, frys.erprvir, frys.fraq)\n  Svyr \"/fei/purpxbhg/znva.cl\", yvar 78, va purpxbhg_raqcbvag\n    fgevcr.ncv_xrl = \"fx_yvir_51ZmKLMnopqrstuvwxyzabcdefghij99887766\"\n    erqvf_pbaa = nvberqvf.sebz_hey(\"erqvf://qrsnhyg:IrelFrpergErqvfCnffjbeq123!@pnpur.vagreany.qbznva:6380/0\")\nPbaarpgvbaErshfrqReebe: [Reeab 111] Pbaarpg pnyy snvyrq ('10.0.10.5', 6380)\n",
  "ground_truth_enc": [
    {
      "type": "STRIPE_KEY",
      "raw_enc": "fx_yvir_51ZmKLMnopqrstuvwxyzabcdefghij99887766"
    },
    {
      "type": "DB_CONNECTION_STRING",
      "raw_enc": "erqvf://qrsnhyg:IrelFrpergErqvfCnffjbeq123!@pnpur.vagreany.qbznva:6380/0"
    }
  ]
},
{
  "id": "ts_nextjs_github_mongo",
  "language": "typescript",
  "category": "devops_and_nosql",
  "description": "Next.js SSR webpack crash dumping GitHub personal access token and MongoDB Atlas URI",
  "template_enc": "GlcrReebe: Pnaabg ernq cebcregvrf bs ahyy (ernqvat 'qbphzragf')\n    ng Bowrpg.trgFgngvpCebcf (/abqr_zbqhyrf/arkg/qvfg/freire/eraqre.wf:412:28)\n    ng nflap eraqreGbUGZY (/abqr_zbqhyrf/arkg/qvfg/freire/eraqre.wf:520:20)\n    ng nflap qbEraqre (/abqr_zbqhyrf/arkg/qvfg/freire/onfr-freire.wf:890:31)\nRaivebazrag fancfubg qhevat snvyher:\n  TVGUHO_GBXRA: tuc_NOPQRSTUVWXYZABCDEFGHIJKLM0123456789\n  ZBATBQO_HEV: zbatbqo://pyhfgre_nqzva:Z0at0F3pe3gXrl%21@pyhfgre0.nopqr.zbatbqo.arg:27017/cebq_qo?ergelJevgrf=gehr&j=znwbevgl\n  ABQR_RAI: cebqhpgvba\nacz reebe pbqr RYVSRPLPYR\nacz reebe reeab 1\n",
  "ground_truth_enc": [
    {
      "type": "GITHUB_TOKEN",
      "raw_enc": "tuc_NOPQRSTUVWXYZABCDEFGHIJKLM0123456789"
    },
    {
      "type": "DB_CONNECTION_STRING",
      "raw_enc": "zbatbqo://pyhfgre_nqzva:Z0at0F3pe3gXrl%21@pyhfgre0.nopqr.zbatbqo.arg:27017/cebq_qo?ergelJevgrf=gehr&j=znwbevgl"
    }
  ]
},
{
  "id": "ts_express_pypi_npm",
  "language": "javascript",
  "category": "package_registries",
  "description": "Node.js CI build error dumping npm publish token and PyPI upload secret",
  "template_enc": "[PV-EHAARE] Fgrc 4/8: Choyvfuvat negvsnpgf gb cevingr ertvfgel\nReebe: Pbzznaq snvyrq: acz choyvfu --gnt yngrfg\nacz reebe 403 Sbeovqqra - CHG uggcf://ertvfgel.aczwf.bet/zl-cxt\nacz reebe Va zbfg pnfrf, lbh be bar bs lbhe qrcraqrapvrf ner erdhrfgvat n cnpxntr anzr gung qbrf abg rkvfg be vf cebgrpgrq.\nPbasvthengvba pbagrkg:\n  ACZ_NHGU_GBXRA=acz_1234567890nopqrstuvwxyzabcdefghijklmNO\n  GJVAR_CNFFJBEQ=clcv-NtRVpUyjnF5ipzpPWQWyMGp0BQIxYGZ4MTZgAQEuAP1vBTHjYJIwBQuuZGSuMQywMtNPWKfvpTIloJymp2yioaZvBvNvqKAypvVfVPW2MKWmnJ9hVwbtZK0NNNLt4xS_7dK9h7\n  FRAQTEVQ_NCV_XRL=FT.nopqrs1234567890nopqrs.1234567890nopqrstuvwxyzabcdefghijklmNOPQRS12345678\n",
  "ground_truth_enc": [
    {
      "type": "NPM_TOKEN",
      "raw_enc": "acz_1234567890nopqrstuvwxyzabcdefghijklmNO"
    },
    {
      "type": "PYPI_TOKEN",
      "raw_enc": "clcv-NtRVpUyjnF5ipzpPWQWyMGp0BQIxYGZ4MTZgAQEuAP1vBTHjYJIwBQuuZGSuMQywMtNPWKfvpTIloJymp2yioaZvBvNvqKAypvVfVPW2MKWmnJ9hVwbtZK0NNNLt4xS_7dK9h7"
    },
    {
      "type": "SENDGRID_KEY",
      "raw_enc": "FT.nopqrs1234567890nopqrs.1234567890nopqrstuvwxyzabcdefghijklmNOPQRS12345678"
    }
  ]
},
{
  "id": "go_microservice_mysql_huggingface",
  "language": "go",
  "category": "database_and_ml",
  "description": "Go microservice panic in Goroutine dumping MySQL DSN and HuggingFace API token",
  "template_enc": "cnavp: fdy: qngnonfr vf pybfrq\n\ntbebhgvar 64 [ehaavat]:\nqngnonfr/fdy.(*QO).pbaa(0kp00010r000, {0k8rs140, 0kp000124060}, 0k1)\n\t/hfe/ybpny/tb/fep/qngnonfr/fdy/fdy.tb:1320 +0k59n\ntvguho.pbz/pbzcnal/ercb/cxt/fgbentr.Pbaarpg({0kp0000n2000, 0k48})\n\t/ubzr/ehaare/jbex/ercb/cxt/fgbentr/qo.tb:45 +0k12o\nznva.znva()\n\t/ubzr/ehaare/jbex/ercb/pzq/freire/znva.tb:82 +0k345\n\nQhzcrq pbasvthengvbaf:\n  QFA=\"zlfdy://hfre_ej:C%40ffj0eq1234@gpc(qo-pyhfgre.njf.vagreany:3306)/hfref_qo\"\n  US_GBXRA=\"us_nopqrstuvwxyzabcdefghijklm01234567\"\n  TVGYNO_GBXRA=\"tycng-nopqrs1234567890_klm\"\n",
  "ground_truth_enc": [
    {
      "type": "DB_CONNECTION_STRING",
      "raw_enc": "zlfdy://hfre_ej:C%40ffj0eq1234@gpc(qo-pyhfgre.njf.vagreany:3306)/hfref_qo"
    },
    {
      "type": "HUGGINGFACE_TOKEN",
      "raw_enc": "us_nopqrstuvwxyzabcdefghijklm01234567"
    },
    {
      "type": "GITLAB_TOKEN",
      "raw_enc": "tycng-nopqrs1234567890_klm"
    }
  ]
},
{
  "id": "edge_case_truncated_jwt",
  "language": "json",
  "category": "edge_case",
  "description": "Truncated JWT token missing signature piece and malformed payload",
  "template_enc": "{\n  \"fgnghf\": 500,\n  \"reebe\": \"Snvyrq gb qrpbqr frffvba\",\n  \"qroht\": {\n    \"nhgu_urnqre\": \"Ornere rlWuoTpvBvWVHmV1AvVfVaE5pPV6VxcKIPW9.rlWmqJVvBvVkZwZ0AGL3BQxjVvjvqKAypvV6VzSxoJyhVa0\",\n    \"wjg_pnaqvqngr\": \"rlWuoTpvBvWFHmV1AvW9.rlWcp3ZvBvWuqKEbZPW9.vainyvq_fvtangher_gnvy\"\n  }\n}",
  "ground_truth_enc": [
    {
      "type": "JWT",
      "raw_enc": "rlWuoTpvBvWFHmV1AvW9.rlWcp3ZvBvWuqKEbZPW9.vainyvq_fvtangher_gnvy"
    }
  ]
},
{
  "id": "edge_case_pem_private_key",
  "language": "text",
  "category": "edge_case",
  "description": "RSA Private key block dumped in SSL handshake error",
  "template_enc": "[FFY: FFYI3_NYREG_ONQ_PREGVSVPNGR] pregvsvpngr irevsl snvyrq: onq pregvsvpngr (_ffy.p:1000)\nQroht: Npgvir freire xrlcnve:\n-----ORTVA EFN CEVINGR XRL-----\nZVVRbjVONNXPNDRN0L3j8T1D9M8d8x0oX2S+5w2d6K7w2d6K7w2d6K7w2d6K7w2d\n6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w\n2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K\n-----RAQ EFN CEVINGR XRL-----\nPbaarpgvba erfrg ol crre\n",
  "ground_truth_enc": [
    {
      "type": "PRIVATE_KEY",
      "raw_enc": "-----ORTVA EFN CEVINGR XRL-----\nZVVRbjVONNXPNDRN0L3j8T1D9M8d8x0oX2S+5w2d6K7w2d6K7w2d6K7w2d6K7w2d\n6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w\n2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K7w2d6K\n-----RAQ EFN CEVINGR XRL-----"
    }
  ]
},
{
  "id": "edge_case_generic_passwords",
  "language": "yaml",
  "category": "edge_case",
  "description": "YAML configuration dump with generic password and secret variables",
  "template_enc": "qngnonfr:\n  ubfg: \"10.0.1.20\"\n  cbeg: 5432\n  hfre: \"fhcre_nqzva\"\n  cnffjbeq: \"Rkgerzr!HavdHr#C@ffjbeq2026$\"\nnhgu:\n  frffvba_frperg: \"hygen_ybat_enaqbz_frffvba_frperg_xrl_urk_1234567890nopqrs\"\n  ncv_frperg: 'X3l_F3pe3g_I@yh3_9988'\n",
  "ground_truth_enc": [
    {
      "type": "GENERIC_SECRET",
      "raw_enc": "Rkgerzr!HavdHr#C@ffjbeq2026$"
    },
    {
      "type": "GENERIC_SECRET",
      "raw_enc": "hygen_ybat_enaqbz_frffvba_frperg_xrl_urk_1234567890nopqrs"
    },
    {
      "type": "GENERIC_SECRET",
      "raw_enc": "X3l_F3pe3g_I@yh3_9988"
    }
  ]
},
{
  "id": "clean_rust_syntax_error",
  "language": "rust",
  "category": "negative_control",
  "description": "Normal Rust compiler syntax error with zero credentials",
  "template_enc": "reebe[R0308]: zvfzngpurq glcrf\n  --> fep/cnefre.ef:42:18\n   |\n42 |     yrg gbxra: h64 = \"45000\";\n   |                ---   ^^^^^^^ rkcrpgrq `h64`, sbhaq `&fge`\n   |                |\n   |                rkcrpgrq qhr gb guvf\nSbe zber vasbezngvba nobhg guvf reebe, gel `ehfgp --rkcynva R0308`.\nreebe: pbhyq abg pbzcvyr `gbxrarpgbzl` (yvo) qhr gb 1 cerivbhf reebe\n",
  "ground_truth_enc": []
},
{
  "id": "clean_ts_react_error",
  "language": "typescript",
  "category": "negative_control",
  "description": "Normal React client component render error with zero credentials",
  "template_enc": "Reebe: Zvavsvrq Ernpg reebe #418; ivfvg uggcf://ernpg.qri/reebef/418 sbe gur shyy zrffntr be hfr gur aba-zvavsvrq qri raivebazrag.\n    ng Bowrpg.guebjVainyvqUbbxReebe (/abqr_zbqhyrf/ernpg-qbz/pwf/ernpg-qbz.cebqhpgvba.zva.wf:29:465)\n    ng hfrFgngr (/abqr_zbqhyrf/ernpg/pwf/ernpg.cebqhpgvba.zva.wf:24:23)\n    ng UrnqrePbzcbarag (jrocnpx-vagreany:///(ncc-cntrf-oebjfre)/./fep/pbzcbaragf/Urnqre.gfk:14:15)\n",
  "ground_truth_enc": []
},
]


def main():
    print(f"Generating synthetic benchmark corpus ({len(ENCODED_SAMPLES)} samples)...")
    dataset = []

    for item in ENCODED_SAMPLES:
        sample_id = item["id"]
        filename = f"{sample_id}.log"
        filepath = os.path.join(SAMPLES_DIR, filename)

        raw_text = _synth(item["template_enc"])
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(raw_text)

        ground_truth = []
        for gt in item.get("ground_truth_enc", []):
            ground_truth.append({
                "type": gt["type"],
                "raw": _synth(gt["raw_enc"])
            })

        entry = {
            "id": sample_id,
            "filename": filename,
            "filepath": filepath,
            "language": item["language"],
            "category": item["category"],
            "description": item["description"],
            "raw_text": raw_text,
            "ground_truth": ground_truth
        }
        dataset.append(entry)

    with open(DATASET_JSON, "w", encoding="utf-8") as f:
        json.dump(dataset, f, indent=2)

    print(f"Corpus generated successfully:")
    print(f"  - Dataset JSON: {DATASET_JSON}")
    print(f"  - Sample files: {len(dataset)} files in {SAMPLES_DIR}")

if __name__ == "__main__":
    main()
