package com.finova.reactnative;

import com.facebook.react.ReactPackage;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.uimanager.ViewManager;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * Finova Network React Native Package
 * Entry point for Finova SDK integration with React Native
 */
public class FinovaReactNativePackage implements ReactPackage {
    
    @Override
    public List<NativeModule> createNativeModules(ReactApplicationContext reactContext) {
        List<NativeModule> modules = new ArrayList<>();
        modules.add(new FinovaModule(reactContext));
        modules.add(new MiningModule(reactContext));
        modules.add(new XPModule(reactContext));
        modules.add(new ReferralModule(reactContext));
        modules.add(new NFTModule(reactContext));
        modules.add(new WalletModule(reactContext));
        modules.add(new SocialModule(reactContext));
        modules.add(new AnalyticsModule(reactContext));
        return modules;
    }

    @Override
    public List<ViewManager> createViewManagers(ReactApplicationContext reactContext) {
        return Collections.emptyList();
    }
}
