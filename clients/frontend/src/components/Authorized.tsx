import { ICPSnapApi, SignRawMessageResponse } from "@astrox/icsnap-types"
import React, { useEffect, useRef, useState } from "react"
import { CreateActorResult, getWalletConnect } from "../services/connection"
import { FOX, LOGO } from "../utils/resCont"
import { _SERVICE as factory_SERVICE } from "../candid/foxic_factory"
import { _SERVICE as wallet_SERVICE } from "../candid/foxic_wallet"
import { Principal } from "@dfinity/principal"
import { hasOwnProperty } from "../utils"
import FOX_ANIMATE from "../assets/FOXIC.json"
import Lottie from "react-lottie"
import Wallet from "./Wallet"
import { SnapIdentity } from "@astrox/icsnap-adapter"
import Copy from "./Copy"

const defaultOptions = {
  loop: true,
  autoplay: true,
  animationData: FOX_ANIMATE,
  rendererSettings: {
    preserveAspectRatio: "xMidYMid slice",
  },
}
type AuthorizedProps = {
  identity: SnapIdentity
  // api: ICPSnapApi
  factoryConnect: CreateActorResult<factory_SERVICE>
}

const Authorized: React.FC<AuthorizedProps> = (props) => {
  const { identity, factoryConnect } = props
  const [message, setMessage] = useState<string | undefined>(undefined)
  const [signedMessage, setSignedMessage] = useState<
    SignRawMessageResponse | undefined
  >(undefined)
  const [walletCanister, setWalletCanister] = useState<Principal>()
  const [walletController, setWalletController] = useState<[Principal] | []>()
  const [walletConnect, setWalletConnect] =
    useState<CreateActorResult<wallet_SERVICE>>()
  const [createLoading, setCreateLoading] = useState(false)

  useEffect(() => {
    initWallet()
  }, [factoryConnect])

  const signMessage = async () => {
    console.log('identity', identity)
    console.log('identity', identity.signRawMessage)
    const signed = await identity.signRawMessage(message!)
    console.log({ signed })
    setSignedMessage(signed)
  }

  const create = async () => {
    setCreateLoading(true)
    try {
      const uploadResult = await factoryConnect.actor?.factory_wallet_install()
      console.log(uploadResult)
      if (uploadResult && hasOwnProperty(uploadResult, "Ok")) {
        const { controller, canister_id } = uploadResult.Ok
        setWalletCanister(canister_id)
        setWalletController([controller])
        const res = await getWalletConnect(identity, canister_id.toText())
        setWalletConnect(res)
        setCreateLoading(false)
      } else {
        console.warn(uploadResult)
      }
    } catch (error) {
      console.log("install error", error)
      setCreateLoading(false)
    }
  }

  const initWallet = async () => {
    console.log("factoryConnect", factoryConnect)
    if (factoryConnect) {
      try {
        const resWallets = await factoryConnect.actor?.get_wallet()
        console.log(resWallets)
        if (resWallets && resWallets.length > 0 && resWallets[0]) {
          const { canister_id, controller } = resWallets[0]
          setWalletCanister(canister_id)
          setWalletController(controller)
          // wallet = await getActor(identity, WalletIDL, wallet_canister.toText());
          const res = await getWalletConnect(identity, canister_id.toText())
          setWalletConnect(res)
        }
      } catch (error) {
        console.log(error)
      }
    }
  }

  return (
    <div className="container">
      <div
        className="flex align-items-end"
        style={{ marginTop: 40, marginBottom: 60 }}
      >
        <div style={{ width: 100, height: 76, marginRight: 10 }}>
          <Lottie width={100} height={76} options={defaultOptions} />
        </div>
        <div>
          <img src={LOGO} style={{ width: 92, height: 26 }} alt="" />
          <p style={{ color: "#B0B0B0" }}>
            MetaMask Flask wallet that allows you to send ICP and sign messages.
          </p>
        </div>
      </div>
      <div className="flex justify-between">
        <div className="card">
          <h2>Principal ID</h2>
          <div className="flex" style={{ marginBottom: 30 }}>
            <p className="c_grey">{identity?.getPrincipal().toText()}</p>
            <Copy text={identity?.getPrincipal().toText()} content={<a className="c_brand">COPY</a>} />
          </div>
          <input
            aria-label="To Sign a message"
            style={{ padding: "1em" }}
            placeholder="To Sign a message"
            onChange={(e) => {
              setMessage(e.target.value)
            }}
          />
          <a className="button-primary button-block" onClick={signMessage}>
            Sign Message
          </a>
        </div>
        <div className="card">
          <h2>Signature</h2>
          <div className="flex">
            <p>{signedMessage?.signature}</p>
          </div>
        </div>
      </div>
      <div className="flex justify-between">
        {walletConnect ? (
          <Wallet
            walletCanister={walletCanister?.toText()!}
            walletController={walletController!}
            walletConnect={walletConnect!}
          />
        ) : createLoading ? (
          <div className="card">
            {createLoading ? <h2>Creating…</h2> : null}
          </div>
        ) : (
          <div style={{ width: 400 }}>
            <h1>Use MetaMask Flask to create Identity.</h1>
            <a className="button-primary" onClick={create}>
              Create
            </a>
          </div>
        )}
      </div>
    </div>
  )
}

export default Authorized
