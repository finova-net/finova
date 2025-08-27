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
 * Main package registration for all Finova native modules
 * 
 * @version 1.0.0
 * @author Finova Team
 */
public class FinovaReactNativePackage implements ReactPackage {
    
    @Override
    public List<NativeModule> createNativeModules(ReactApplicationContext reactContext) {
        List<NativeModule> modules = new ArrayList<>();
        
        // Core modules
        modules.add(new FinovaClientModule(reactContext));
        modules.add(new WalletConnectorModule(reactContext));
        modules.add(new TransactionManagerModule(reactContext));
        
        // Service modules
        modules.add(new MiningServiceModule(reactContext));
        modules.add(new XPServiceModule(reactContext));
        modules.add(new ReferralServiceModule(reactContext));
        modules.add(new NFTServiceModule(reactContext));
        
        // Utility modules
        modules.add(new BiometricAuthModule(reactContext));
        modules.add(new CryptoUtilsModule(reactContext));
        modules.add(new NetworkStatusModule(reactContext));
        modules.add(new DeviceInfoModule(reactContext));
        
        return modules;
    }
    
    @Override
    public List<ViewManager> createViewManagers(ReactApplicationContext reactContext) {
        return Collections.emptyList();
    }
}
